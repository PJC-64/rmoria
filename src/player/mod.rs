#[derive(Debug, Clone)]
pub struct Player {
    pub name: String,
    pub x: usize,
    pub y: usize,
    pub max_hp: i32,
    pub hp: i32,
}

impl Player {
    pub fn new(name: &str, x: usize, y: usize) -> Self {
        Self {
            name: name.to_string(),
            x,
            y,
            max_hp: 20,
            hp: 20,
        }
    }

    pub fn move_to(&mut self, x: usize, y: usize) {
        self.x = x;
        self.y = y;
    }
}
