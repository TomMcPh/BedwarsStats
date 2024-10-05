use reqwest::blocking::get;
use reqwest::Error;
use serde_json::Value;
use cursive::Cursive;
use cursive::views::{Dialog, TextArea, LinearLayout, Button, TextView};
use cursive::view::Nameable;
use cursive::CursiveExt;
use cursive::view::Resizable;
use cursive::theme::{Color, Effect, Style};
use cursive::utils::markup::StyledString;
use cursive::theme::BaseColor;
use base64::{engine::general_purpose, Engine};
extern crate csv;
use std::fs::OpenOptions;
use csv::Writer;
use csv::Reader;



/*TODO:
- Use the texture to somehow create a mc face.
- Allow "Enter" keystroke to move the page.
- Set mvp++ and mvp+ ranks depending on hypixel level
*/

const API_KEY: &str = "b89896a1-7020-4880-9182-271b24f95970";


/* Rough reference from Plancke.io */
fn get_level_for_exp(exp: i64) -> u32 {
    let exp: u32 = exp as u32;
    const EASY_LEVELS_XP: [u32; 4] = [500, 1000, 2000, 3500];
    const XP_PER_LEVEL: u32 = 5000;
    const LEVELS_PER_PRESTIGE: u32 = 100;

    // Calculate the XP required for one prestige.
    let mut total_xp_per_prestige = 0;
    for xp in &EASY_LEVELS_XP {
        total_xp_per_prestige += xp;
    }
    total_xp_per_prestige += (LEVELS_PER_PRESTIGE - EASY_LEVELS_XP.len() as u32) * XP_PER_LEVEL;

    // Calculate the number of prestiges and remaining XP
    let prestiges = exp / total_xp_per_prestige;
    let exp_without_prestiges = exp % total_xp_per_prestige;

    // Calculate the level based on remaining XP
    let mut level = prestiges * LEVELS_PER_PRESTIGE;
    let mut current_xp = exp_without_prestiges;
    
    for xp in EASY_LEVELS_XP.iter() {
        if current_xp >= *xp {
            level += 1;
            current_xp -= *xp;
        } else {
            break;
        }
    }

    level += current_xp / XP_PER_LEVEL;
    level
}

fn hypixelrank(rank: &str) -> StyledString {
    let mut result = StyledString::plain("");
    let mvpplusplus = Color::Rgb(228, 158, 10);
    let mvp = Color::Rgb(85, 255, 255);
    let vip = Color::Rgb(85, 255, 85);
    let unranked = Color::Rgb(42, 42, 42);
    let youtube = Color::Rgb(255, 85, 85);
    let staff = Color::Rgb(255, 85, 85);
    let vipplus = Color::Rgb(251,177,32);
    let mvpplus = Color::Rgb(252,91,91);
    if rank == "SUPERSTAR" {
        let rank = "MVP";
        let charrr= "]";
        let rankplus = "++";
        result.append_styled(rank, Style::from(mvpplusplus).combine(Effect::Bold));
        result.append_styled(rankplus, Style::from(mvpplus).combine(Effect::Bold));
        result.append_styled(charrr, Style::from(mvpplusplus).combine(Effect::Bold));
        result.append_plain("\n");
    }
    if rank == "MVP_PLUS" {
        let charrr= "]";
        let rank = "[MVP";
        let rankplus = "+";
        result.append_styled(rank, Style::from(mvp).combine(Effect::Bold));
        result.append_styled(rankplus, Style::from(mvpplus).combine(Effect::Bold));
        result.append_styled(charrr, Style::from(mvp).combine(Effect::Bold));
        result.append_plain("\n");
    }
    if rank == "MVP" {
        let rank = "MVP";
        result.append_styled(format!("[{}]\n", rank), Style::from(mvp).combine(Effect::Bold));
    }
    if rank == "VIP_PLUS" {
        let rank = "[VIP";
        let rankplus = "+";
        let charrr= "]";
        result.append_styled(rank, Style::from(vip).combine(Effect::Bold));
        result.append_styled(rankplus, Style::from(vipplus).combine(Effect::Bold));
        result.append_styled(charrr, Style::from(vip).combine(Effect::Bold));
        result.append_plain("\n");
    }
    if rank == "VIP" {
        let rank = "VIP";
        result.append_styled(format!("[{}]\n", rank), Style::from(vip).combine(Effect::Bold));
    }
    if rank == "YOUTUBER" {
        let rank = "YOUTUBE";
        result.append_styled(format!("[{}]\n", rank), Style::from(youtube).combine(Effect::Bold));
    }
    if rank == "ADMIN" {
        let rank = "ADMIN";
        result.append_styled(format!("[{}]\n", rank), Style::from(staff).combine(Effect::Bold));
    }
    if rank == "Not Available" {
        let rank = "Unranked";
        result.append_styled(format!("[{}]\n", rank), Style::from(unranked).combine(Effect::Bold));
    }

    result
}


fn statcolor(stat_value: f64) -> Color {
    let iron = Color::Rgb(66, 66, 66);
    let gold = Color::Rgb(241, 196, 15 );
    let diamond = Color::Rgb(52, 152, 219 );
    let ruby = Color::Rgb(185, 2, 5);
    let crystal = Color::Rgb(70, 12, 164);

    match stat_value {
        0.0..=0.01 => ruby,
        0.02..=1.0 => iron,
        1.1..=2.0 => iron,
        2.1..=3.9 => gold,
        4.0..=10.0 => diamond,
        10.1..=25.0 => crystal,
        _ => ruby, 
    }
}

fn levelcolor(stat_value: f64) -> Color {
    let iron = Color::Rgb(66, 66, 66);
    let gold = Color::Rgb(241, 196, 15 );
    let diamond = Color::Rgb(52, 152, 219 );
    let emerald = Color::Rgb(34, 153, 84);
    let sapphire = Color::Rgb(191, 201, 202);
    let ruby = Color::Rgb(183, 28, 28);
    let crystal = Color::Rgb(216, 27, 96);
    let opal = Color::Rgb(13, 71, 161);
    let amethyst = Color::Rgb(74, 20, 140);
    let thousand = Color::Rgb(51, 0, 102);

    match stat_value {
        1.0..=100.0 => iron,
        100.1..=199.9 => iron,
        200.0..=299.9 => gold,
        300.0..=399.9 => diamond,
        400.0..=499.9 => emerald,
        500.0..=599.9 => sapphire,
        600.0..=699.9 => ruby,
        700.0..=799.9 => crystal,
        800.0..=899.9 => opal,
        900.0..=999.9 => amethyst,
        _ => thousand, 
    }
}

fn bedwarsstats(stats: &Value, username: &str) -> StyledString {

    let bedwars = stats.get("Bedwars");
    let mut result = StyledString::plain("");
    let black = Color::Rgb(0, 0, 0);
    let dark_green = Color::Rgb(61,131,61);


    let exp = bedwars
        .and_then(|b| b.get("Experience"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let stars = get_level_for_exp(exp);
    let levelcolour = levelcolor(stars.into());
    result.append_plain("Stars: ");
    result.append_styled(format!("{}✫\n", stars), Style::from(levelcolour).combine(Effect::Bold));

    let games_played = bedwars
        .and_then(|b| b.get("games_played_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Games played: ");
    result.append_styled(format!("{}\n", games_played), Style::from(black).combine(Effect::Bold));
    let formatted_games_played = format!("{:.2}", games_played);

    let wins_bedwars = bedwars
        .and_then(|b| b.get("wins_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Games won: ");
    result.append_styled(format!("{}\n", wins_bedwars), Style::from(dark_green));
    let formatted_wins_bedwars = format!("{:.2}", wins_bedwars);

    let losses_bedwars = bedwars
        .and_then(|b| b.get("losses_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    result.append_plain("Games lost: ");
    result.append_styled(format!("{}\n", losses_bedwars), Style::from(Color::Light(BaseColor::Red)));
    let formatted_losses_bedwars = format!("{:.2}", losses_bedwars);

    let wlr: f64 = wins_bedwars as f64 / losses_bedwars as f64;
    let statcolour = statcolor(wlr);
    result.append_plain("WLR: ");
    result.append_styled(format!("{:.2}\n", wlr), Style::from(statcolour).combine(Effect::Bold));

    let kills_bedwars = bedwars
        .and_then(|b| b.get("kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Total kills: ");
    result.append_styled(format!("{}\n", kills_bedwars), Style::from(dark_green));
    let formatted_kills_bedwars = format!("{:.2}", kills_bedwars);

    let deaths_bedwars = bedwars
        .and_then(|b| b.get("deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    result.append_plain("Total deaths: ");
    result.append_styled(format!("{}\n", deaths_bedwars), Style::from(Color::Light(BaseColor::Red)));
    let formatted_deaths_bedwars = format!("{:.2}", deaths_bedwars);

    let kdr: f64 = kills_bedwars as f64 / deaths_bedwars as f64;
    let statcolour = statcolor(kdr);
    result.append_plain("KDR: ");
    result.append_styled(format!("{:.2}\n", kdr), Style::from(statcolour).combine(Effect::Bold));

    let final_kills = bedwars
        .and_then(|b| b.get("final_kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Final kills: ");
    result.append_styled(format!("{}\n", final_kills), Style::from(dark_green));
    let formatted_final_kills = format!("{:.2}", final_kills);

    let final_deaths = bedwars
        .and_then(|b| b.get("final_deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    result.append_plain("Final deaths: ");
    result.append_styled(format!("{}\n", final_deaths), Style::from(Color::Light(BaseColor::Red)));
    let formatted_final_deaths = format!("{:.2}", final_deaths);

    let fkdr: f64 = final_kills as f64 / final_deaths as f64;
    let statcolour = statcolor(fkdr);
    result.append_plain("FKDR: ");
    result.append_styled(format!("{:.2}\n", fkdr), Style::from(statcolour).combine(Effect::Bold));

    let beds_broken = bedwars
        .and_then(|b| b.get("beds_broken_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Beds broken: ");
    result.append_styled(format!("{}\n", beds_broken), Style::from(dark_green));
    let formatted_beds_broken = format!("{:.2}", beds_broken);

    let beds_lost = bedwars
        .and_then(|b| b.get("beds_lost_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    result.append_plain("Beds lost: ");
    result.append_styled(format!("{}\n", beds_lost), Style::from(Color::Light(BaseColor::Red)));
    let formatted_beds_lost = format!("{:.2}", beds_lost);

    let bblr = beds_broken as f64 / beds_lost as f64;
    let statcolour = statcolor(bblr);
    result.append_plain("BBLR: ");
    result.append_styled(format!("{:.2}\n", bblr), Style::from(statcolour).combine(Effect::Bold));
    let formatted_bblr = format!("{:.2}", bblr);

    let file = OpenOptions::new()
        .append(true)
        .open("/Users/thomas/Desktop/Projects/HypixelStats/session.csv").expect("Error reading file");

    let mut writer = Writer::from_writer(file);
    
    writer.write_record(&[username, " ", &formatted_games_played, &formatted_wins_bedwars, &formatted_losses_bedwars, " ", &formatted_final_kills, &formatted_final_deaths, " ", &formatted_kills_bedwars, &formatted_deaths_bedwars, " ", &formatted_beds_broken, &formatted_beds_lost, " "]).expect("Error writing stats");

    writer.flush().expect("Error flushing");

    result
}

fn additionalstats(stats: &Value) -> StyledString {
    let dark_green = Color::Rgb(17, 103, 20);
    let ironcolor = Color::Rgb(130, 130, 130);
    let goldcolor = Color::Rgb(155, 108, 10);
    let diamondcolor = Color::Rgb(85, 255, 255);
    let emeraldcolor = Color::Rgb(0, 98, 0);
    let bedwars = stats.get("Bedwars");
    let mut result = StyledString::plain("");
    let winstreak = bedwars
        .and_then(|b| b.get("winstreak"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Winstreak: ");
    let statcolour = statcolor(winstreak as f64);
    result.append_styled(format!("{}\n", winstreak), Style::from(statcolour).combine(Effect::Bold));

    let single_fks = bedwars
        .and_then(|b| b.get("eight_one_final_kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let single_fds = bedwars
        .and_then(|b| b.get("eight_one_final_deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);
    let single_fkdr: f64 = single_fks as f64 / single_fds as f64;
    let statcolour = statcolor(single_fkdr);
    result.append_plain("Singles FKDR: ");
    result.append_styled(format!("{:.2}\n", single_fkdr), Style::from(statcolour).combine(Effect::Bold));

    let double_fks = bedwars
        .and_then(|b| b.get("eight_two_final_kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let double_fds = bedwars
        .and_then(|b| b.get("eight_two_final_deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);

    let double_fkdr: f64 = double_fks as f64 / double_fds as f64;
    let statcolour = statcolor(double_fkdr);
    result.append_plain("Doubles FKDR: ");
    result.append_styled(format!("{:.2}\n", double_fkdr), Style::from(statcolour).combine(Effect::Bold));

    let trios_fks = bedwars
        .and_then(|b| b.get("four_three_final_kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let trios_fds = bedwars
        .and_then(|b| b.get("four_three_final_deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);

    let trios_fkdr: f64 = trios_fks as f64 / trios_fds as f64;
    let statcolour = statcolor(trios_fkdr);
    result.append_plain("3v3v3v3 FKDR: ");
    result.append_styled(format!("{:.2}\n", trios_fkdr), Style::from(statcolour).combine(Effect::Bold));

    let quads_fks = bedwars
        .and_then(|b| b.get("four_four_final_kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let quads_fds = bedwars
        .and_then(|b| b.get("four_four_final_deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);

    let quads_fkdr: f64 = quads_fks as f64 / quads_fds as f64;
    let statcolour = statcolor(quads_fkdr);
    result.append_plain("4v4v4v4 FKDR: ");
    result.append_styled(format!("{:.2}\n", quads_fkdr), Style::from(statcolour).combine(Effect::Bold));

    let fourvfour_fks = bedwars
        .and_then(|b| b.get("two_four_final_kills_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let fourvfour_fds = bedwars
        .and_then(|b| b.get("two_four_final_deaths_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(1);

    let fourvfour_fkdr: f64 = fourvfour_fks as f64 / fourvfour_fds as f64;
    let statcolour = statcolor(fourvfour_fkdr);
    result.append_plain("4v4 FKDR: ");
    result.append_styled(format!("{:.2}\n", fourvfour_fkdr), Style::from(statcolour).combine(Effect::Bold));
    
    result.append_plain(" "); 
    result.append_plain("\n");

    let coins = bedwars
        .and_then(|b| b.get("coins"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Tokens: ");
    result.append_styled(format!("{}\n", coins), Style::from(dark_green));

    let iron = bedwars
        .and_then(|b| b.get("iron_resources_collected_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Iron: ");
    result.append_styled(format!("{}\n", iron), Style::from(ironcolor));

    let gold = bedwars
        .and_then(|b| b.get("gold_resources_collected_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Gold: ");
    result.append_styled(format!("{}\n", gold), Style::from(goldcolor));

    let diamond = bedwars
        .and_then(|b| b.get("diamond_resources_collected_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Diamonds: ");
    result.append_styled(format!("{}\n", diamond), Style::from(diamondcolor));

    let emerald = bedwars
        .and_then(|b| b.get("emerald_resources_collected_bedwars"))
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    result.append_plain("Emerald: ");
    result.append_styled(format!("{}\n", emerald), Style::from(emeraldcolor));

    result.append_plain(" "); 
    result.append_plain("\n");

    let line1: &str = r" _  _";
    let line2: &str = r" (o)(o)--.";
    let line3: &str = r"  \../ (  ) written by tom.";
    let line4: &str = r"  m\/m--m'--.";

    result.append_plain(format!("{}\n", line1));
    result.append_plain(format!("{}\n", line2));
    result.append_plain(format!("{}\n", line3));
    result.append_plain(format!("{}\n", line4));
    
    result

}


fn requests(username: &str) -> Result<(StyledString, StyledString, StyledString), Error> {
    let uuid_url = format!("https://api.mojang.com/users/profiles/minecraft/{}", username);
    let response = get(&uuid_url)?;
    if response.status().is_success() {
        let mojang_response: Value = response.json()?;
        if let Some(id) = mojang_response.get("id").and_then(|id| id.as_str()) {    
            let skin_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{}", id); 
            let response = get(&skin_url)?;
            if response.status().is_success() {
                let skin_response: Value = response.json()?;
                let skin_value = skin_response.get("properties")
                .and_then(|p| p.as_array())
                .and_then(|arr| arr.get(0))
                .and_then(|prop| prop.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("Not available");
                let decoded64 = general_purpose::STANDARD.decode(skin_value).unwrap();
                let decodedskin = String::from_utf8(decoded64).unwrap();
                let json_data: Value = serde_json::from_str(&decodedskin).unwrap();
                let url = json_data.get("textures")
                    .and_then(|textures| textures.get("SKIN"))
                    .and_then(|skin| skin.get("url"))
                    .and_then(|url| url.as_str())
                    .unwrap_or("Not available");
            let hypixel_url = format!("https://api.hypixel.net/player?key={}&uuid={}", API_KEY, id);
            let response = get(&hypixel_url)?;
            if response.status().is_success() {
                let hypixel_response: Value = response.json()?;
                if hypixel_response.get("success").and_then(|s| s.as_bool()) == Some(true) {
                    if let Some(player) = hypixel_response.get("player") {
                        if let Some(stats) = player.get("stats") {
                            let specialrank = player.get("rank")
                                .and_then(|r| r.as_str())
                                .unwrap_or("Not Available");
                            let mvpplusplus = player.get("monthlyPackageRank")
                                .and_then(|r| r.as_str())
                                .unwrap_or("Not Available");
                            let mut rank = player.get("newPackageRank")
                                .and_then(|r| r.as_str())
                                .unwrap_or("Not Available");
                            if rank == "MVP_PLUS" {
                                if mvpplusplus == "SUPERSTAR" {
                                    rank = "SUPERSTAR";
                                }
                                if specialrank == "YOUTUBER" {
                                    rank = "YOUTUBER"
                                }
                                if specialrank == "ADMIN" {
                                    rank = "ADMIN";
                                }
                            }
                            return Ok((bedwarsstats(stats, username), additionalstats(stats), hypixelrank(rank)));
                        } else {
                            return Ok((StyledString::plain("Player stats are missing"), StyledString::plain(""), StyledString::plain("Not Available")));
                        }
                    } else {
                        return Ok((StyledString::plain("Player data is missing"), StyledString::plain(""), StyledString::plain("Not Available")));
                    }
                } else {
                    return Ok((StyledString::plain("Error in Hypixel API response"), StyledString::plain(""), StyledString::plain("Not Available")));
                }
            }
        }
    }

    }
    Ok((StyledString::plain("Error fetching UUID (Check your api)."), StyledString::plain(""), StyledString::plain("Not Available")))
}

fn basesession() -> StyledString {
    let mut result = StyledString::plain("");
    /*result.append_plain("Stars Gained: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));*/
    result.append_plain("Games Played: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("Games Won: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));    
    result.append_plain("Games Lost: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("WLR: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("Final Kills: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("Final Deaths: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("FKDR: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("Kills: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("Deaths: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("KDR: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    /* result.append_plain("Beds Broken: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));
    result.append_plain("Beds Lost: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));   
    result.append_plain("BBLR: ");
    result.append_styled(
        format!("{}\n", 0),
        Style::from(Color::Light(BaseColor::Black)));*/


    result
}

fn stat_page(siv: &mut Cursive) {
    let textarea = TextArea::new()
        .with_name("textarea")
        .fixed_width(40);

    siv.add_layer(
        Dialog::new()
            .title("Hypixel Bedwars Stats")
            .content(
                LinearLayout::vertical()
                    .child(textarea)
                    .child(Button::new("Submit", |s| {
                        let username = s.call_on_name("textarea", |view: &mut TextArea| {
                            view.get_content().to_string()
                        }).unwrap();
                        let username_title = username.clone();
                        let username_caps = username_title.to_uppercase();
                        s.pop_layer();

                        // Store the original username in a new variable before using it
                        let original_username = username.clone();

                        match requests(&username) {
                            Ok((bedwars_stats, additional_stats, rank)) => {
                                let mut username_with_rank = StyledString::new();
                                username_with_rank.append(rank);
                                username_with_rank.append_styled(format!(" {}", username_caps), Style::default().combine(Effect::Bold));
                                s.add_layer(
                                    LinearLayout::horizontal()
                                        .child(
                                            Dialog::around(
                                                LinearLayout::vertical()
                                                    .child(Dialog::text(username_with_rank))
                                                    .child(Dialog::text(bedwars_stats))
                                            )
                                            .button("Close", |s| {
                                                s.quit();
                                            })
                                            .button("Search", |s| {
                                                s.pop_layer();
                                                stat_page(s);
                                            })
                                            .button("Session", move |s| {
                                                s.pop_layer();
                                                player_session(s, original_username.clone());
                                            })
                                        )
                                        .child(
                                            Dialog::around(
                                                LinearLayout::vertical()
                                                    .child(Dialog::text(additional_stats))
                                            )
                                        )
                                );
                            }
                            Err(e) => {
                                s.add_layer(
                                    Dialog::text(StyledString::styled(
                                        format!("Error: {}", e),
                                        Color::Light(BaseColor::Red),
                                    ))
                                    .button("Close", |s| {
                                        s.quit();
                                    })
                                );
                            }
                        }
                    }))
            )
            .button("Quit", |s| s.quit())
    );
}

fn readcsv(username: &str) -> StyledString {
    let file = OpenOptions::new()
        .read(true)

    // Create a CSV reader
    let mut rdr = Reader::from_reader(file);

    // Initialize the StyledString
    let mut result = StyledString::plain("");

    // Skip the first row (headers)
    let _ = rdr.records().next();

    // Initialize a variable to hold the first and last record
    let mut first_record: Option<csv::StringRecord> = None;
    let mut last_record: Option<csv::StringRecord> = None;

    // Iterate through all records and keep track of the first and last matching records
    for record in rdr.records() {
        if let Ok(record) = record {
            // Trim whitespaces from the username and CSV value
            if record.get(0).map(|s| s.trim()) == Some(username.trim()) {
                if first_record.is_none() {
                    first_record = Some(record.clone());
                }
                last_record = Some(record);
            }
        }
    }

    // Check if both first and last records are found
    if let (Some(first), Some(last)) = (first_record, last_record) {
        let diff = |f: usize, l: usize| -> i32 {
            let first_val: i32 = first.get(f).unwrap_or("0").trim().parse().unwrap_or(0);
            let last_val: i32 = last.get(l).unwrap_or("0").trim().parse().unwrap_or(0);
            last_val - first_val
        };

        result.append_plain("Games Played: ");
        result.append_styled(format!("{}\n", diff(2, 2)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Games Won: ");
        result.append_styled(format!("{}\n", diff(3, 3)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Games Lost: ");
        result.append_styled(format!("{}\n", diff(4, 4)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("WLR: ");
        result.append_styled(format!("{}\n", diff(5, 5)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Final Kills: ");
        result.append_styled(format!("{}\n", diff(6, 6)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Final Deaths: ");
        result.append_styled(format!("{}\n", diff(7, 7)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("FKDR: ");
        result.append_styled(format!("{}\n", diff(8, 8)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Kills: ");
        result.append_styled(format!("{}\n", diff(9, 9)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Deaths: ");
        result.append_styled(format!("{}\n", diff(10, 10)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("KDR: ");
        result.append_styled(format!("{}\n", diff(11, 11)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Beds Broken: ");
        result.append_styled(format!("{}\n", diff(12, 12)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("Beds Lost: ");
        result.append_styled(format!("{}\n", diff(13, 13)), Style::from(Color::Light(BaseColor::Black)));

        result.append_plain("BBLR: ");
        result.append_styled(format!("{}\n", diff(14, 14)), Style::from(Color::Light(BaseColor::Black)));
    } else {
        result.append_plain("No records found for the username.\n");
    }

    result
}


fn player_session(siv: &mut Cursive, username: String) {
    let username_caps = username.to_uppercase();

    match requests(&username) {
        Ok((bedwars_stats, additional_stats, rank)) => {
            let mut username_with_rank = StyledString::new();
            username_with_rank.append(rank);
            username_with_rank.append_styled(format!(" {}", username_caps), Style::default().combine(Effect::Bold));
            siv.add_layer(
                LinearLayout::horizontal()
                    .child(
                        Dialog::around(
                            LinearLayout::vertical()
                                .child(Dialog::text(username_with_rank))
                                /*.child(Dialog::text(basesession()))*/
                                .child(Dialog::text(readcsv(&username_caps)))
                                /*WORKS*/
                                /*We need to run some checks to tell which is to be run!
                                - Also need to do the maths for the inputs
                                - Specift which user we are looking for*/
                        )
                        .button("Close", |s| {
                            s.quit();
                            s.pop_layer();
                        })
                        .button("Search", |s| {
                            s.pop_layer();
                            stat_page(s);
                        })
                        .button("Session", move |s| {
                            s.pop_layer();
                            //player_session(s, username.clone());
                        })
                    )
                    .child(
                        Dialog::around(
                            LinearLayout::vertical()
                                /*.child(Dialog::text(additional_stats))*/
                        )
                    )
            );
        }
        Err(e) => {
            siv.add_layer(
                Dialog::text(StyledString::styled(
                    format!("Error: {}", e),
                    Color::Light(BaseColor::Red),
                ))
                .button("Close", |s| {
                    s.quit();
                })
            );
        }
    }
}



fn main() {
    /*write_csv();*/
    let mut siv = Cursive::default();
    siv.load_toml(include_str!("/Users/thomas/Desktop/Projects/HypixelStats/Layout.toml")).unwrap();

    stat_page(&mut siv);

    siv.run();
}
