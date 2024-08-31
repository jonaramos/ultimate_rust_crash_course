use std::{
    error::Error, 
    {io, thread}, 
    sync::mpsc::{self, Receiver},
    time::Duration
};
use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand
};
use rusty_audio::Audio;
use invaders::{
    frame::{self, new_frame, Drawable, Frame},
    render
};

fn main() -> Result<(), Box<dyn Error>>
{
    const SOUNDS: &'static str = "sounds/";
    let mut audio = Audio::new();
    audio.add("explode", SOUNDS.to_string() + "explode.wav");
    audio.add("lose", SOUNDS.to_string() + "lose.wav");
    audio.add("move", SOUNDS.to_string() + "move.wav");
    audio.add("pew", SOUNDS.to_string() + "pew.wav");
    audio.add("startup", SOUNDS.to_string() + "startup.wav");
    audio.add("win", SOUNDS.to_string() + "win.wav");
    audio.play("startup");

    // Terminal
    let mut stdout = io::stdout();
    terminal::enable_raw_mode()?;
    stdout.execute(EnterAlternateScreen)?;
    stdout.execute(Hide)?;

    // Render loop in a separate thread
    let (render_tx, render_rx) = mpsc::channel();
    let render_handle = thread::spawn(move || {
        let mut last_frame = frame::new_frame();
        let mut stdout = io::stdout();
        render::render(&mut stdout, &last_frame, &last_frame, true);
        loop {
            let curr_frame = match render_rx.recv() {
                Ok(x) => x,
                Err(_) => break,
            };
            render::render(&mut stdout, &last_frame, &curr_frame,  false);
            last_frame = curr_frame;
        }
    });

    // Game Loop
    'gameloop: loop {
        // Per-frame init
        let curr_frame = new_frame();

        // Input
        while event::poll(Duration::default())? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc | KeyCode::Char('q') => {
                        audio.play("lose");
                        break 'gameloop;
                    }
                    _ => {}
                }
            }
        }

        // Draw & render
        let _ = render_tx.send(curr_frame);
        thread::sleep(Duration::from_millis(1));
    }
    // Cleanup
    drop(render_tx);
    render_handle.join().unwrap();
 
    audio.wait();
    stdout.execute(Show)?;
    stdout.execute(LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    Ok(())
}
