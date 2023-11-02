use std::{collections::VecDeque, error::Error, process::exit, time::Duration};

use macroquad::prelude::*;

use ::rand::{rngs::ThreadRng, thread_rng, Rng};

#[derive(Clone, Copy, PartialEq)]
#[repr(C, align(64))]
struct Pos<const W: u32, const H: u32> {
    x: u32,
    y: u32,
}

impl<const W: u32, const H: u32> Pos<W, H> {
    fn wrapping_inc(n: u32, limit: u32) -> u32 {
        if n == limit - 1 {
            0
        } else {
            n + 1
        }
    }

    fn wrapping_dec(n: u32, limit: u32) -> u32 {
        if n == 0 {
            limit - 1
        } else {
            n - 1
        }
    }

    pub fn up(&mut self) {
        self.y = Self::wrapping_dec(self.y, H);
    }

    pub fn down(&mut self) {
        self.y = Self::wrapping_inc(self.y, H);
    }

    pub fn left(&mut self) {
        self.x = Self::wrapping_dec(self.x, W);
    }

    pub fn right(&mut self) {
        self.x = Self::wrapping_inc(self.x, W);
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum Direction {
    None,
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn opposite(&self) -> Self {
        match self {
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::None => Direction::None,
        }
    }
}

impl TryFrom<KeyCode> for Direction {
    type Error = ();

    fn try_from(value: KeyCode) -> Result<Self, Self::Error> {
        Ok(match value {
            KeyCode::Up => Self::Up,
            KeyCode::Down => Self::Down,
            KeyCode::Left => Self::Left,
            KeyCode::Right => Self::Right,
            _ => return Err(()),
        })
    }
}

#[derive(PartialEq, Eq)]
enum Health {
    Alive,
    Dying,
    Dead,
}

struct Snake<const W: u32, const H: u32> {
    health: Health,
    body: VecDeque<Pos<W, H>>,
    direction: Direction,
    needs_to_grow: bool,
}

impl<const W: u32, const H: u32> Snake<W, H> {
    pub fn new() -> Self {
        Self {
            health: Health::Alive,
            body: vec![Pos { x: W / 2, y: H / 2 }].into(),
            direction: Direction::None,
            needs_to_grow: false,
        }
    }

    pub fn len(&self) -> usize {
        self.body.len()
    }

    pub fn head(&self) -> Pos<W, H> {
        *self.body.front().unwrap()
    }

    pub fn will_collide(&self, head: Pos<W, H>) -> bool {
        for pos in self.body.iter().skip(1).rev().skip(1).copied() {
            if head == pos {
                return true;
            }
        }
        false
    }

    pub fn tick(&mut self, apple: &mut Pos<W, H>, rng: &mut ThreadRng) {
        if self.health == Health::Dead {
            return;
        }

        let mut head = self.head();
        match self.direction {
            Direction::Up => head.up(),
            Direction::Down => head.down(),
            Direction::Left => head.left(),
            Direction::Right => head.right(),
            Direction::None => return,
        }

        if self.will_collide(head) {
            self.health = match self.health {
                Health::Alive => Health::Dying,
                _ => Health::Dead,
            };
            return;
        }

        self.health = Health::Alive;
        self.body.push_front(head);

        if self.contains(*apple) {
            self.needs_to_grow = true;
            while self.contains(*apple) {
                *apple = Pos {
                    x: rng.gen_range(0..W),
                    y: rng.gen_range(0..H),
                };
            }
        }

        if self.health != Health::Dead && !self.needs_to_grow {
            self.body.pop_back();
        }
        self.needs_to_grow = false;
    }

    pub fn contains(&mut self, pos: Pos<W, H>) -> bool {
        self.body.contains(&pos)
    }
}

struct Game<const W: u32, const H: u32> {
    rng: ThreadRng,
    snake: Snake<W, H>,
    apple: Pos<W, H>,
}

impl<const W: u32, const H: u32> Game<W, H> {
    fn new() -> Self {
        let mut rng = thread_rng();
        let mut snake = Snake::new();
        let mut apple = snake.head();

        while snake.contains(apple) {
            apple = Pos {
                x: rng.gen_range(0..W),
                y: rng.gen_range(0..H),
            };
        }

        Self { rng, snake, apple }
    }
}

#[macroquad::main("snek")]
async fn main() -> Result<(), Box<dyn Error>> {
    const W: u32 = 20;
    const H: u32 = 10;

    let mut game = Game::<W, H>::new();
    let mut events = VecDeque::with_capacity(8);

    const POLLING_RATE: Duration = Duration::from_millis(200);
    let mut last_update = get_time();

    fn draw_cell(x: u32, y: u32, color: Color) {
        draw_rectangle(
            x as f32 / W as f32 * screen_width(),
            y as f32 / H as f32 * screen_height(),
            1.0 / W as f32 * screen_width(),
            1.0 / H as f32 * screen_height(),
            color,
        );
    }

    loop {
        let now = get_time();

        let head = game.snake.head();
        draw_cell(head.x, head.y, GREEN);
        for (idx, Pos { x, y }) in game.snake.body.iter().skip(1).enumerate() {
            draw_cell(
                *x,
                *y,
                Color::new(0.0, 0.6 - 0.1 * (idx & 1) as f32, 0.19, 1.00),
            );
        }
        draw_cell(game.apple.x, game.apple.y, RED);

        for x in 0..W {
            draw_line(
                x as f32 / W as f32 * screen_width(),
                0.0,
                x as f32 / W as f32 * screen_width(),
                screen_height(),
                2.0,
                DARKGRAY,
            );
        }

        for y in 0..H {
            draw_line(
                0.0,
                y as f32 / H as f32 * screen_height(),
                screen_width(),
                y as f32 / H as f32 * screen_height(),
                2.0,
                DARKGRAY,
            );
        }

        if game.snake.health == Health::Dead {
            let text = format!("You died with a length of {}.", game.snake.len());
            draw_rectangle(
                0.0,
                0.0,
                text.len() as f32 * 15.0,
                105.0,
                Color::new(0.3, 0.3, 0.3, 0.7),
            );
            draw_text(&text, 10.0, 30.0, 30.0, ORANGE);
            draw_text("Press 'Q' to quit.", 10.0, 60.0, 30.0, ORANGE);
            draw_text("Press 'R' to restart.", 10.0, 90.0, 30.0, ORANGE);
        }

        for key in [KeyCode::Up, KeyCode::Down, KeyCode::Left, KeyCode::Right] {
            if is_key_pressed(key) {
                events.push_front((now, key));
            }
        }

        if is_key_pressed(KeyCode::R) {
            game = Game::<W, H>::new();
            events.clear();
            next_frame().await;
            continue;
        }

        if is_key_pressed(KeyCode::Q) {
            exit(0);
        }

        if now - last_update > POLLING_RATE.as_secs_f64() {
            let cur_dir = game.snake.direction;
            while let Some((when, k)) = events.pop_back() {
                if now - when > POLLING_RATE.as_secs_f64() * 3.0 {
                    continue;
                }
                match k {
                    KeyCode::Up | KeyCode::Down | KeyCode::Left | KeyCode::Right => {
                        let new_dir = Direction::try_from(k).unwrap();
                        if !(game.snake.len() > 1 && cur_dir == new_dir.opposite())
                            && cur_dir != new_dir
                        {
                            game.snake.direction = new_dir;
                            if !game.snake.will_collide(head) {
                                break;
                            }
                        }
                    }
                    _ => (),
                }
            }

            game.snake.tick(&mut game.apple, &mut game.rng);

            last_update = now;
        }

        next_frame().await;
    }
}
