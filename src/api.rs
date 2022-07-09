use image::RgbaImage;
use serde::{Deserialize, Serialize};
use worker::{Error, Fetch};

pub async fn fetch_skyblock_profile(player: &Player, key: String) -> Result<SkyblockProfile, Error> {
    let url = format!("https://api.altpapier.dev/v1/profiles/{id}?key={key}", id = player.id, key = key);
    let mut res = Fetch::Url(url.parse().unwrap()).send().await?;
    let profiles_res = (res.json().await as Result<SkyblockProfilesResponse, Error>)?;
    if profiles_res.status != 200 {
        return Err(Error::from(format!("failed to get profile: {}", profiles_res.status)));
    }

    let mut profiles = profiles_res.data.unwrap();
    profiles.sort_by_key(|p| p.last_save);
    Ok(profiles.last().expect("no profiles found").clone())
}

pub async fn fetch_lily_weight(player: &Player, key: String) -> Result<SkyblockLilyWeight, Error> {
    // We have to use the raw worker URL if we want to host this on our domain.
    // TODO automatically swap between these two.
    // let url = format!("https://lily.antonio32a.com/{id}?key={key}", id = player.id, key = key);
    let url = format!("https://lilyweight.antonio32a.workers.dev/{id}?key={key}", id = player.id, key = key);
    let mut res = Fetch::Url(url.parse().unwrap()).send().await?;
    let weight_res = (res.json().await as Result<SkyblockLilyWeightResponse, Error>)?;
    if !weight_res.success {
        return Err(Error::from(format!("failed to get lily weight: {}", res.status_code())));
    }
    Ok(weight_res.data.unwrap())
}

pub async fn fetch_player(username: String) -> Result<Player, Error> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username);
    let mut res = Fetch::Url(url.parse().unwrap()).send().await?;
    let player = (res.json().await as Result<Player, Error>)?;
    Ok(player)
}

pub async fn fetch_head(player: &Player) -> Result<RgbaImage, Error> {
    let url = format!("https://crafthead.net/avatar/{}/50", player.id);
    let mut res = Fetch::Url(url.parse().unwrap()).send().await?;
    let img = image::load_from_memory(&res.bytes().await?);
    Ok(img
        .expect("failed to get DynamicImage from ImageResult")
        .as_rgba8()
        .expect("rgba8 image is null")
        .to_owned()
    )
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Player {
    pub name: String,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkyblockProfilesResponse {
    pub status: u16,
    pub data: Option<Vec<SkyblockProfile>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockProfile {
    pub username: String,
    pub id: String,
    pub name: String,
    // pub isIronman: bool,
    pub last_save: u64,
    pub fairy_souls: u16,
    pub networth: SkyblockNetworth,
    pub weight: SkyblockSenitherWeight,
    pub skills: SkyblockSkills,
    pub dungeons: Option<SkyblockDungeons>,
    pub slayer: SkyblockSlayers,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockNetworth {
    pub no_inventory: Option<bool>,
    pub total_networth: Option<f64>,
    pub purse: Option<f64>,
    pub bank: Option<f64>,
    // pub types
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockSenitherWeight {
    pub total_weight: f32,
    pub total_weight_with_overflow: f32,
    // pub weight
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockSkills {
    pub farming: SkyblockSkill,
    pub mining: SkyblockSkill,
    pub combat: SkyblockSkill,
    pub foraging: SkyblockSkill,
    pub fishing: SkyblockSkill,
    pub enchanting: SkyblockSkill,
    pub alchemy: SkyblockSkill,
    pub carpentry: SkyblockSkill,
    pub runecrafting: SkyblockSkill,
    pub social: SkyblockSkill,
    pub taming: SkyblockSkill,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct SkyblockSkill {
    pub xp: u64,
    pub level: u16,
    pub xpCurrent: u64,
    pub xpForNext: u64,
    pub progress: f32,
    pub levelWithProgress: Option<f32>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockDungeons {
    pub selected_class: Option<String>,
    pub secrets_found: u32,
    // pub classes
    pub catacombs: SkyblockCatacombs,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockCatacombs {
    pub skill: SkyblockSkill,
    pub highest_tier_completed: Option<String>,
    // pub floors
    // pub master_mode_floors
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockSlayers {
    pub zombie: SkyblockSlayer,
    pub spider: SkyblockSlayer,
    pub wolf: SkyblockSlayer,
    pub enderman: SkyblockSlayer,
    pub blaze: SkyblockSlayer,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[allow(non_snake_case)]
pub struct SkyblockSlayer {
    pub xp: u64,
    pub level: u16,
    pub xpForNext: u64,
    pub progress: f32,
    // pub kills
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SkyblockLilyWeightResponse {
    pub success: bool,
    pub data: Option<SkyblockLilyWeight>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SkyblockLilyWeight {
    pub uuid: String,
    pub total: f32,
    // pub skill
    // pub catacombs
    pub slayer: f32,
}
