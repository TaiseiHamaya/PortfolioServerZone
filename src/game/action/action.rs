use nalgebra::Point3;

pub struct ActionAcquired {
    job_name: String,
    level: u32,
}

enum ActionType {
    UNSPECIFIED,
    WEAPONSKILL,
    ABILITY,
    SPELL,
}

enum TargetType {
    MySelf,
    Player,
    Enemy,
    SelfRange,
    TargetRange,
}

enum TargetValue {
    MySelf,
    Player(u64),
    Enemy(u64),
    SelfRange(f32),
    TargetRange(Point3<f32>, f32),
}

pub struct EntityAction {
    id: u32,
    name: String,
    acquired: ActionAcquired,
    action_type: ActionType,
    cast: chrono::TimeDelta,
    damage_delay: chrono::TimeDelta,
    recast: chrono::TimeDelta,
    mp_cost: u32,
    range: f32,
    radius: Option<f32>,
}

impl EntityAction {
    pub fn new() -> Self {
        EntityAction {
            id: 0,
            name: "Holy Spirit".to_string(),
            acquired: ActionAcquired {
                job_name: "Paladin".to_string(),
                level: 64,
            },
            action_type: ActionType::SPELL,
            cast: chrono::TimeDelta::from_std(std::time::Duration::from_secs_f32(1.5)).unwrap(),
            damage_delay: chrono::TimeDelta::from_std(std::time::Duration::from_secs_f32(0.2)).unwrap(),
            recast: chrono::TimeDelta::from_std(std::time::Duration::from_secs_f32(2.5)).unwrap(),
            mp_cost: 1000,
            range: 25.0f32,
            radius: None,
        }
    }

    pub fn cast_time(&self) -> chrono::TimeDelta {
        self.cast
    }

    pub fn recast_time(&self) -> chrono::TimeDelta {
        self.recast
    }
}

pub struct CurrentAction {
    id: u32,
    start_time: chrono::DateTime<chrono::Utc>,
    
}

trait EntityActionTrait {}
