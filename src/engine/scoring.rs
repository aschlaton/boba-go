use crate::engine::models::{CardKind, Player};

/// Score contribution from a category
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CategoryScore {
    pub category: String,
    pub points: f32,
}

/// Complete score breakdown for a player
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreBreakdown {
    pub category_scores: Vec<CategoryScore>,
    pub set_bonuses: Vec<SetBonus>,
    pub total_score: f32,
}

/// Set bonus (e.g., 3 unique teas = +5)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SetBonus {
    pub description: String,
    pub points: f32,
}

impl ScoreBreakdown {
    pub fn new() -> Self {
        Self {
            category_scores: Vec::new(),
            set_bonuses: Vec::new(),
            total_score: 0.0,
        }
    }
}

pub fn calculate_player_score(
    player: &Player,
    all_players: &[Player],
    player_id: usize,
) -> ScoreBreakdown {
    let mut breakdown = ScoreBreakdown::new();
    let mut total_score: f32 = 0.0;
    
    let (base_points, mut base_breakdown) = score_base_categories(player);
    total_score += base_points;
    breakdown.category_scores.append(&mut base_breakdown);
    
    let (boost_points, mut boost_breakdown) = score_boosted_fruit_teas(player);
    total_score += boost_points;
    breakdown.category_scores.append(&mut boost_breakdown);

    if let Some(mochi) = score_mochi_ice_cream(player) {
        total_score += mochi.points;
        breakdown.category_scores.push(mochi);
    }

    if let Some(tapioca) = score_tapioca_pearl(all_players, player_id) {
        total_score += tapioca.points;
        breakdown.category_scores.push(tapioca);
    }

    if let Some(tea_set_bonus) = score_tea_set_bonus(player) {
        total_score += tea_set_bonus.points;
        breakdown.category_scores.push(tea_set_bonus);
    }
    
    breakdown.total_score = total_score;
    breakdown
}

fn score_base_categories(player: &Player) -> (f32, Vec<CategoryScore>) {
    let mut total: f32 = 0.0;
    let mut categories: Vec<CategoryScore> = Vec::new();

    for (card_kind, count) in &player.public_cards {
        if *count == 0 {
            continue;
        }
        if *card_kind == CardKind::MochiIceCream {
            continue;
        }

        let points_per_card = card_kind.score() as f32;
        let points = points_per_card * (*count as f32);
        total += points;
        categories.push(CategoryScore { category: card_kind.name().to_string(), points });
    }

    (total, categories)
}

fn score_boosted_fruit_teas(player: &Player) -> (f32, Vec<CategoryScore>) {
    let mut total: f32 = 0.0;
    let mut categories: Vec<CategoryScore> = Vec::new();

    for (card_kind, count) in &player.boosted_fruit_teas {
        if *count == 0 {
            continue;
        }
        let points_per_card = card_kind.score() as f32;
        let boosted_points = (points_per_card * 3.0) * (*count as f32);
        total += boosted_points;
        categories.push(CategoryScore { category: format!("{} (boosted)", card_kind.name()), points: boosted_points });
    }

    (total, categories)
}

fn score_mochi_ice_cream(player: &Player) -> Option<CategoryScore> {
    let count = player.public_cards.get(&CardKind::MochiIceCream).copied().unwrap_or(0);
    if count == 0 {
        return None;
    }

    let capped = count.min(5);
    let points: u32 = match capped {
        1 => 1,
        2 => 3,
        3 => 6,
        4 => 10,
        _ => 15,
    } as u32;

    Some(CategoryScore { category: CardKind::MochiIceCream.name().to_string(), points: points as f32 })
}

fn score_tapioca_pearl(all_players: &[Player], player_id: usize) -> Option<CategoryScore> {
    let n = all_players.len();
    if n == 0 { return None; }

    let counts: Vec<usize> = all_players.iter()
        .map(|p| p.public_cards.get(&CardKind::TapiocaPearl).copied().unwrap_or(0))
        .collect();

    let my_count = counts[player_id];
    let &max_c = counts.iter().max()?;
    let &min_c = counts.iter().min()?;

    let max_ties = counts.iter().filter(|&&c| c == max_c).count();
    let min_ties = counts.iter().filter(|&&c| c == min_c).count();

    let mut points: f32 = 0.0;
    if my_count == max_c && max_c > 0 {
        points += 6.0 / (max_ties as f32);
    }
    if my_count == min_c {
        points -= 6.0 / (min_ties as f32);
    }

    if points.abs() < f32::EPSILON { return None; }
    Some(CategoryScore { category: "Tapioca Pearl Majority/Minority".to_string(), points })
}

fn score_tea_set_bonus(player: &Player) -> Option<CategoryScore> {
    let counts = [
        player.public_cards.get(&CardKind::ThaiTea).copied().unwrap_or(0),
        player.public_cards.get(&CardKind::Matcha).copied().unwrap_or(0),
        player.public_cards.get(&CardKind::BrownSugarMilkTea).copied().unwrap_or(0),
        player.public_cards.get(&CardKind::MysteryTea).copied().unwrap_or(0),
    ];

    let mut counts_vec = counts.to_vec();
    counts_vec.sort_unstable_by(|a, b| b.cmp(a));
    let most = counts_vec.get(0).copied().unwrap_or(0);
    let second = counts_vec.get(1).copied().unwrap_or(0);
    let third = counts_vec.get(2).copied().unwrap_or(0);
    let fourth = counts_vec.get(3).copied().unwrap_or(0);

    let sets = most.min(second).min(third + fourth);
    if sets == 0 {
        return None;
    }

    let points = (sets as f32) * 5.0;
    Some(CategoryScore { category: "Tea Set Bonus".to_string(), points })
}

