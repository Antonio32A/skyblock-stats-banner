extern crate core;

use image::{Rgba, RgbaImage};
use indexmap::IndexMap;
use num_format::{Locale, ToFormattedString};
use regex::Regex;
use rusttype::{Font, Scale};
use unwrap_or::unwrap_ok_or;
use worker::*;

use crate::api::{Player, SkyblockCatacombs, SkyblockDungeons, SkyblockLilyWeight, SkyblockProfile, SkyblockSkill};
use crate::utils::handle_error;
use crate::utils::string_width;

mod utils;
mod api;

const BLACK: Rgba<u8> = Rgba([0, 0, 0, 255]);
const LIGHT_GRAY: Rgba<u8> = Rgba([137, 137, 137, 255]);
const DARK_GRAY: Rgba<u8> = Rgba([100, 100, 100, 255]);
const WATERMARK: &str = "skyblock-stats.antonio32a.com";

#[event(fetch)]
pub async fn main(global_req: Request, env: Env, _ctx: Context) -> Result<Response> {
    utils::set_panic_hook();

    Router::new()
        .get("/", |_req, _ctx| {
            Response::redirect("https://github.com/Antonio32A/skyblock-stats-banner".parse().unwrap())
        })
        .get_async("/:name", |req, ctx| async move {
            let name = ctx.param("name")
                .expect("failed to get name param")
                .to_owned();

            if !Regex::new(r"^\w{1,16}$").unwrap().is_match(&name) {
                return Response::error("Bad Request", 400);
            }

            let player = unwrap_ok_or!(
                api::fetch_player(name).await,
                err, return handle_error(err, "Could not fetch player", 400)
            );

            let (
                profile_res,
                lily_weight_res,
                head_res
            ) = futures::future::join3(
                api::fetch_skyblock_profile(&player, ctx.var("ALTPAPIER_KEY")?.to_string()),
                api::fetch_lily_weight(&player, ctx.var("HYPIXEL_KEY")?.to_string()),
                api::fetch_head(&player),
            ).await;

            let profile = unwrap_ok_or!(profile_res, err, return handle_error(err, "Could not fetch player's profile", 500));
            let lily_weight = unwrap_ok_or!(lily_weight_res, err, return handle_error(err, "Could not fetch player's lily weight", 500));
            let head = unwrap_ok_or!(head_res, err, return handle_error(err, "Could not fetch player's head", 500));

            let mut img = create_image(player, profile, lily_weight, head);
            let user_agent = req.headers().get("user-agent")
                .unwrap_or(Some("".to_string()))
                .unwrap_or("".to_string());

            // Resize the image when used as a  Hypixel forum signature so it fits.
            if user_agent == "XenForo/2.x (https://hypixel.net)" {
                img = image::imageops::resize(
                    &img,
                    (800.0 / 1.35) as u32,
                    (400.0 / 1.35) as u32,
                    image::imageops::FilterType::Nearest,
                );
            }

            utils::image_response(img)
        })
        .run(global_req, env)
        .await
}

fn create_image(player: Player, profile: SkyblockProfile, lily_weight: SkyblockLilyWeight, head: RgbaImage) -> RgbaImage {
    let font_data = include_bytes!("../assets/Minecraftia-Regular.ttf");
    let font = Font::try_from_bytes(font_data).expect("failed to load font");

    let img_data = include_bytes!("../assets/template.png");
    let mut img = image::load_from_memory(img_data)
        .expect("failed to get DynamicImage from ImageResult")
        .as_rgba8()
        .expect("rgba8 image is null")
        .to_owned();

    // head, name and uuid
    image::imageops::overlay(&mut img, &head, 20, 20);
    img = imageproc::drawing::draw_text(
        &img,
        BLACK,
        80, 20,
        Scale::uniform(30.0),
        &font,
        &player.name,
    );
    imageproc::drawing::draw_text_mut(
        &mut img,
        LIGHT_GRAY,
        80, 50,
        Scale::uniform(20.0),
        &font,
        &player.id,
    );

    // profile name and watermark
    imageproc::drawing::draw_text_mut(
        &mut img,
        DARK_GRAY,
        800 - 15 - string_width(&font, &profile.name, Scale::uniform(30.0)) as i32,
        20,
        Scale::uniform(30.0),
        &font,
        &profile.name,
    );
    imageproc::drawing::draw_text_mut(
        &mut img,
        LIGHT_GRAY,
        800 - 15 - string_width(&font, &WATERMARK.to_string(), Scale::uniform(20.0)) as i32,
        50,
        Scale::uniform(20.0),
        &font,
        &WATERMARK.to_string(),
    );

    // networth and weights
    let networth: String;
    if profile.networth.total_networth.is_some() {
        networth = (profile.networth.total_networth.unwrap() as i64).to_formatted_string(&Locale::en)
    } else {
        networth = "No Inventory API".to_string();
    }

    // This is a bit messy, but should be fine.
    let dungeons = profile.dungeons.unwrap_or(SkyblockDungeons {
        selected_class: None,
        secrets_found: 0,
        catacombs: SkyblockCatacombs {
            skill: SkyblockSkill {
                xp: 0,
                level: 0,
                xpCurrent: 0,
                xpForNext: 0,
                progress: 0.0,
                levelWithProgress: None,
            },
            highest_tier_completed: None,
        },
    });

    let texts = IndexMap::from([
        ("Networth", networth),
        ("Secrets", dungeons.secrets_found.to_formatted_string(&Locale::en)),
        ("Senither Weight", format!(
            "{total} ({overflow})",
            total = (profile.weight.total_weight as i32).to_formatted_string(&Locale::en),
            overflow = (profile.weight.total_weight_with_overflow as i32).to_formatted_string(&Locale::en)
        )),
        ("Lily Weight", (lily_weight.total as i32).to_formatted_string(&Locale::en)),
    ]);

    let mut index = 0;
    for (name, value) in texts {
        imageproc::drawing::draw_text_mut(
            &mut img,
            BLACK,
            20,
            100 + 75 * index,
            Scale::uniform(30.0),
            &font,
            &name,
        );

        imageproc::drawing::draw_text_mut(
            &mut img,
            DARK_GRAY,
            20,
            100 + 30 + 75 * index,
            Scale::uniform(30.0),
            &font,
            &value,
        );
        index += 1;
    }

    let skills = IndexMap::from([
        ((520, 105), profile.skills.taming.level),
        ((630, 105), profile.skills.farming.level),
        ((740, 105), profile.skills.carpentry.level),
        ((520, 165), profile.skills.mining.level),
        ((630, 165), profile.skills.combat.level),
        ((740, 165), profile.skills.runecrafting.level),
        ((520, 225), profile.skills.foraging.level),
        ((630, 225), profile.skills.fishing.level),
        ((740, 225), profile.skills.social.level),
        ((520, 285), profile.skills.enchanting.level),
        ((630, 285), profile.skills.alchemy.level),
        ((740, 285), dungeons.catacombs.skill.level)
    ]);

    for (position, level) in skills {
        let text = level.to_string();
        let text_width = string_width(&font, &text, Scale::uniform(30.0)) as i32;

        imageproc::drawing::draw_text_mut(
            &mut img,
            DARK_GRAY,
            position.0 + (40 - text_width) / 2, // center
            position.1,
            Scale::uniform(30.0),
            &font,
            &text,
        );
    }

    let skills: Vec<u16> = vec![
        profile.skills.taming.level,
        profile.skills.farming.level,
        profile.skills.mining.level,
        profile.skills.combat.level,
        profile.skills.foraging.level,
        profile.skills.fishing.level,
        profile.skills.enchanting.level,
        profile.skills.alchemy.level,
    ];
    let sa: f32 = skills.iter().sum::<u16>() as f32 / skills.len() as f32;
    let sa_text = format!("Skill Average: {:.2}", sa);
    let sa_text_width = string_width(&font, &sa_text, Scale::uniform(30.0)) as i32;
    imageproc::drawing::draw_text_mut(
        &mut img,
        DARK_GRAY,
        460 + (320 - sa_text_width) / 2, // center
        345,
        Scale::uniform(30.0),
        &font,
        &sa_text,
    );

    let slayer = IndexMap::from([
        ((350, 105), profile.slayer.zombie.xp),
        ((350, 165), profile.slayer.spider.xp),
        ((350, 225), profile.slayer.wolf.xp),
        ((350, 285), profile.slayer.enderman.xp),
        ((350, 345), profile.slayer.blaze.xp),
    ]);

    for (position, xp) in slayer {
        let text: String;
        if xp >= 1_000_000 {
            text = format!("{:.1}M", xp as f64 / 1_000_000.0)
        } else if xp >= 1_000 {
            text = format!("{:.1}K", xp as f64 / 1_000.0)
        } else {
            text = format!("{:.1}", xp as f64)
        }

        let text_width = string_width(&font, &text, Scale::uniform(30.0)) as i32;
        imageproc::drawing::draw_text_mut(
            &mut img,
            DARK_GRAY,
            position.0 + (100 - text_width) / 2, // center
            position.1,
            Scale::uniform(30.0),
            &font,
            &text,
        );
    }

    img
}
