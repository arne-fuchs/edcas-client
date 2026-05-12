use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

use crate::event_shim::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use wasm_bindgen::prelude::*;
use web_sys::window;

use crate::app::App;
use crate::wasm_backend::WasmBackend;

thread_local! {
    static KEY_QUEUE: RefCell<VecDeque<KeyEvent>> = RefCell::new(VecDeque::new());
    static TERM_SIZE: RefCell<(u16, u16)>         = RefCell::new((120, 40));
}

/// Called by xterm.js onKey handler to push a keyboard event into Rust.
#[wasm_bindgen]
pub fn push_key_event(key: &str, ctrl: bool, shift: bool, _alt: bool) {
    if let Some(ev) = dom_key_to_crossterm(key, ctrl, shift) {
        KEY_QUEUE.with(|q| q.borrow_mut().push_back(ev));
    }
}

/// Called by xterm.js onResize handler.
#[wasm_bindgen]
pub fn on_resize(cols: u16, rows: u16) {
    TERM_SIZE.with(|s| *s.borrow_mut() = (cols, rows));
}

fn dom_key_to_crossterm(key: &str, ctrl: bool, shift: bool) -> Option<KeyEvent> {
    let modifiers = if ctrl {
        KeyModifiers::CONTROL
    } else if shift {
        KeyModifiers::SHIFT
    } else {
        KeyModifiers::NONE
    };

    let code = match key {
        "Enter"     => KeyCode::Enter,
        "Backspace" => KeyCode::Backspace,
        "Delete"    => KeyCode::Delete,
        "Escape"    => KeyCode::Esc,
        "Tab"       => if shift { KeyCode::BackTab } else { KeyCode::Tab },
        "ArrowUp"   => KeyCode::Up,
        "ArrowDown" => KeyCode::Down,
        "ArrowLeft" => KeyCode::Left,
        "ArrowRight"=> KeyCode::Right,
        "Home"      => KeyCode::Home,
        "End"       => KeyCode::End,
        "PageUp"    => KeyCode::PageUp,
        "PageDown"  => KeyCode::PageDown,
        "F1"        => KeyCode::F(1),
        "F2"        => KeyCode::F(2),
        s if s.chars().count() == 1 => {
            let c = s.chars().next().unwrap();
            KeyCode::Char(c)
        }
        _ => return None,
    };

    Some(KeyEvent::new(code, modifiers))
}

#[wasm_bindgen(start)]
pub fn web_main() {
    console_error_panic_hook::set_once();

    let (cols, rows) = TERM_SIZE.with(|s| *s.borrow());
    let backend = WasmBackend::new(cols, rows);
    let terminal = Rc::new(RefCell::new(
        Terminal::new(backend).expect("failed to create terminal"),
    ));

    let app = Rc::new(RefCell::new(App::new_web()));

    schedule_raf(app, terminal);
}

fn schedule_raf(
    app: Rc<RefCell<App>>,
    terminal: Rc<RefCell<Terminal<WasmBackend>>>,
) {
    let closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let closure_clone = Rc::clone(&closure);

    *closure.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        // Update terminal size if it changed
        let (cols, rows) = TERM_SIZE.with(|s| *s.borrow());
        {
            let mut term = terminal.borrow_mut();
            let current = term.backend_mut();
            current.set_size(cols, rows);
        }

        // Drain key events
        let keys: Vec<KeyEvent> = KEY_QUEUE.with(|q| q.borrow_mut().drain(..).collect());
        for key in keys {
            app.borrow_mut().handle_key(&key);
        }

        // Poll async search results
        app.borrow_mut().poll_search_results();

        // Render
        {
            let mut term = terminal.borrow_mut();
            let mut app_ref = app.borrow_mut();
            let _ = term.draw(|f| app_ref.render(f));
        }

        // Schedule next frame
        if let Some(win) = window() {
            let cb = closure_clone.borrow();
            if let Some(f) = cb.as_ref() {
                let _ = win.request_animation_frame(f.as_ref().unchecked_ref());
            }
        }
    }) as Box<dyn FnMut()>));

    // Kick off first frame
    if let Some(win) = window() {
        let cb = closure.borrow();
        if let Some(f) = cb.as_ref() {
            let _ = win.request_animation_frame(f.as_ref().unchecked_ref());
        }
    }
}
