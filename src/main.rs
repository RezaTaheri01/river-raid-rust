// src means source
use rand::{thread_rng, Rng};
use std::{
    cmp::Ordering::*,
    io::{stdout, Stdout, Write},
    time::Duration,
};
use std::{thread, time};

use crossterm::style::Stylize;
use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{poll, read, Event, KeyCode},
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear},
    ExecutableCommand, QueueableCommand,
};

#[derive(PartialEq, Eq)]
enum PlayerStatus {
    Dead,
    Alive,
    Animation,
    Paused,
}

struct Enemy {
    c: u16,
    l: u16,
}

struct Bullet {
    c: u16,
    l: u16,
    energy: u16,
}

struct World {
    player_c: u16,
    player_l: u16,
    map: Vec<(u16, u16)>,
    maxc: u16,
    maxl: u16,
    status: PlayerStatus,
    next_right: u16,
    next_left: u16,
    ship: String,
    enemy: Vec<Enemy>,
    bullet: Vec<Bullet>,
}

impl World {
    fn new(maxc: u16, maxl: u16) -> World {
        World {
            player_c: maxc / 2,
            player_l: maxl - 1,
            map: vec![(maxc / 2 - 5, maxc / 2 + 5); maxl as usize],
            maxc,
            maxl,
            status: PlayerStatus::Alive,
            next_left: maxc / 2 - 7,
            next_right: maxc / 2 + 7,
            ship: 'P'.to_string(),
            enemy: vec![],
            bullet: vec![],
        }
    }
}

fn draw(mut sc: &Stdout, world: &World) -> std::io::Result<()> {
    // after draw use world give it back :)
    // sc.queue(Clear(ClearType::All))?;
    for l in 0..world.map.len() {
        // jungle
        sc.queue(MoveTo(0, l as u16))?;
        sc.queue(Print(
            "⁞".repeat(world.map[l].0 as usize).green().on_green(),
        ))?;
        sc.queue(MoveTo(world.map[l].1, l as u16))?;
        sc.queue(Print(
            "⁞"
                .repeat((world.maxc - world.map[l].1) as usize)
                .green()
                .on_green(),
        ))?;
        // river
        sc.queue(MoveTo(world.map[l].0, l as u16))?;
        sc.queue(Print(
            "r".repeat((world.map[l].1 - world.map[l].0) as usize)
                .blue()
                .on_blue(),
        ))?;
    }
    // draw player
    sc.queue(MoveTo(world.player_c, world.player_l))?;
    sc.queue(Print("P".red().on_red()))?;
    sc.flush()?;

    Ok(())
}

fn physics(mut world: World) -> std::io::Result<World> {
    let mut rng = thread_rng();

    // check if player died
    if world.player_c < world.map[world.player_l as usize].0
        || world.player_c >= world.map[world.player_l as usize].1
    {
        world.status = PlayerStatus::Dead;
    }

    for l in (0..world.map.len() - 1).rev() {
        world.map[l + 1] = world.map[l];
    }

    let (left, right) = &mut world.map[0];
    match world.next_left.cmp(left) {
        Greater => *left += 1,
        Less => *left -= 1,
        Equal => {}
    };
    match world.next_right.cmp(right) {
        Greater => *right += 1,
        Less => *right -= 1,
        Equal => {}
    };

    const MIN_GAP: u16 = 5; // Define a minimum gap between the left and right boundaries

    // Update left boundary
    if world.next_left == world.map[0].0 && rng.gen_range(0..10) >= 7 {
        world.next_left = rng.gen_range(world.next_left.saturating_sub(5)..world.next_left + 5);
    }

    // Update right boundary
    if world.next_right == world.map[0].1 && rng.gen_range(0..10) >= 7 {
        world.next_right = rng.gen_range(world.next_right.saturating_sub(5)..world.next_right + 5);
    }

    // Ensure there's always a minimum gap between left and right
    if world.next_right <= world.next_left + MIN_GAP {
        // Push right further to maintain the gap
        world.next_right = world.next_left + MIN_GAP;
    }

    // if world.next_right.abs_diff(world.next_left) < 3 {
    //     world.next_right += 3;
    // }

    Ok(world) // return world
}

fn main() -> std::io::Result<()> {
    // using functions (macro)
    let mut sc = stdout(); // mutable !!!
    let (maxc, maxl) = size().unwrap();

    sc.queue(Clear(crossterm::terminal::ClearType::All))?;
    sc.execute(Hide)?;
    sc.flush()?;

    enable_raw_mode()?;
    // init the world
    let slowness = 100;
    let mut world = World::new(maxc, maxl);

    // draw
    while world.status == PlayerStatus::Alive {
        // ready and apply keyboard
        if poll(Duration::from_millis(10))? {
            let key = read().unwrap();
            while poll(Duration::from_millis(0)).unwrap() {
                let _ = read();
            }
            // It's guaranteed that the `read()` won't block when the `poll()`
            // function returns `true`
            match key {
                Event::Key(event) => match event.code {
                    KeyCode::Esc => {
                        break;
                    }
                    KeyCode::Up => {
                        if world.player_l > 1 {
                            world.player_l -= 1
                        };
                    }
                    KeyCode::Left => {
                        if world.player_c > 1 {
                            world.player_c -= 1
                        };
                    }
                    KeyCode::Down => {
                        if world.player_l < maxl - 1 {
                            world.player_l += 1
                        };
                    }
                    KeyCode::Right => {
                        if world.player_c < maxc - 1 {
                            world.player_c += 1
                        };
                    }
                    _ => {}
                },
                _ => {}
            }
        } else {
            // Timeout expired and no `Event` is available
        }

        // physics
        world = physics(world).unwrap();
        // draw
        draw(&sc, &world)?;

        thread::sleep(time::Duration::from_millis(slowness));
    }

    sc.queue(Clear(crossterm::terminal::ClearType::All))?;
    sc.queue(MoveTo(maxc / 2 - 10, maxl / 2))?;
    sc.queue(Print("Thanks For Playing.\n"))?;
    thread::sleep(time::Duration::from_millis(3000));
    sc.queue(Clear(crossterm::terminal::ClearType::All))?;
    sc.execute(Show)?;
    Ok(())
}
