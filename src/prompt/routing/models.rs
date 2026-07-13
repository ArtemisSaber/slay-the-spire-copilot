#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PathCounts {
    pub monsters: usize,
    pub elites: usize,
    pub events: usize,
    pub shops: usize,
    pub rests: usize,
    pub treasures: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ShopTiming {
    #[default]
    None,
    Early,
    Mid,
    Late,
}

impl ShopTiming {
    pub(crate) fn label(self) -> &'static str {
        match self {
            ShopTiming::None => "none",
            ShopTiming::Early => "early",
            ShopTiming::Mid => "mid",
            ShopTiming::Late => "late",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PathMetrics {
    pub counts: PathCounts,
    pub shop_timing: ShopTiming,
    pub rest_before_first_elite: bool,
    pub double_elite_without_rest: bool,
    pub max_elite_to_rest_risk: f64,
}

pub struct PathDescription {
    pub counts: String,
    pub route_chain: String,
    pub annotations: Vec<String>,
    pub metrics: PathMetrics,
}
