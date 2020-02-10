use crate::vg;
use crate::vg::*;
use glfw::{Action, ClientApiHint, Key, WindowEvent, WindowHint, WindowMode, Context};
use std::sync::mpsc::Receiver;
use std::sync::{Arc};
use vulkano::instance::Instance;
use vulkano::swapchain::Surface;
use image::{DynamicImage, imageops};
use std::ops::Deref;

pub struct WindowThing {
    pub events: Receiver<(f64, WindowEvent)>,
    pub surface: Arc<Surface<WrappedWindow>>,
}

impl WindowThing {
    pub fn init_window(instance: Arc<Instance>, dimensions: [u32; 2]) -> WindowThing {
        let mut glfw = glfw::init(glfw::FAIL_ON_ERRORS).unwrap();
        {
            glfw.window_hint(WindowHint::ClientApi(ClientApiHint::NoApi));
            glfw.window_hint(WindowHint::Resizable(true));
            glfw.window_hint(WindowHint::Visible(true));
        }

        // TODO: window, surface, and events in one call? What does this mean?
        let (window, events) = create_window(
            glfw,
            dimensions[0],
            dimensions[1],
            "Pokemon",
            WindowMode::Windowed,
        );

        // Create a surface for the window
        let surface = vg::create_window_surface(instance.clone(), window).unwrap();

        let window = surface.window();
        {
            let mut mut_window = window.write().unwrap();
            mut_window.set_key_polling(true);
            mut_window.set_scroll_polling(true);
            mut_window.set_char_polling(true);
            mut_window.set_size_limits(dimensions[0], dimensions[1], std::u32::MAX, std::u32::MAX);

            if let DynamicImage::ImageRgba8(icon) = image::open("res/master16.png").unwrap() {
                //Set the icon to be multiple sizes of the same icon to account for scaling
                mut_window.set_icon(vec![
                    imageops::resize(&icon, 16, 16, image::imageops::Nearest),
                    imageops::resize(&icon, 32, 32, image::imageops::Nearest),
                    imageops::resize(&icon, 48, 48, image::imageops::Nearest)
                ]);
            }
        }

        WindowThing {
            events,
            surface,
        }
    }

    pub fn handle_input(&self) {
        self.surface.window().poll_events();
        let events = &self.events;
        for (_, event) in glfw::flush_messages(&events) {
            match event {
                glfw::WindowEvent::Key(Key::Escape, _, Action::Press, _) => {
                    let mut window = self.surface.window().write().unwrap();
                    window.set_should_close(true);
                }
                glfw::WindowEvent::Key(Key::Right, _, Action::Press, _) => {
                    // println!("right was pressed");
                    let mut window = self.surface.window().write().unwrap();
                    let (w, h) = window.get_size();
                    window.set_size(w * 2, h * 2);
                }
                glfw::WindowEvent::Key(Key::Left, _, Action::Press, _) => {
                    // println!("left was pressed");
                    let mut window = self.surface.window().write().unwrap();
                    let (w, h) = window.get_size();
                    window.set_size(w / 2, h / 2);
                }
                glfw::WindowEvent::Key(Key::Enter, _, Action::Press, _) => {
                    // println!("enter was pressed");
                    let window = self.surface.window();
                    if window.is_fullscreen() {
                        window.exit_fullscreen()
                    } else {
                        window.enter_fullscreen()
                    }
                }
                glfw::WindowEvent::Key(Key::Down, _, Action::Press, _) => {
                    // println!("enter was pressed");
                    let mut window = self.surface.window().write().unwrap();
                    let (w, h) = window.get_size();
                    window.set_decorated(false);
                    window.set_size(w, h);
                    let (l,u,_,_) = window.get_frame_size();
                    let (x, y) = window.get_pos();
                    window.set_pos(x+l, y+u);
                }
                glfw::WindowEvent::Key(Key::Up, _, Action::Press, _) => {
                    // println!("enter was pressed");
                    let window = self.surface.window();

                    if window.is_fullscreen() { return }

                    {
                        let window = window.read().unwrap();
                        if window.is_decorated() { return }
                    }

                    window.save_attributes();
                    {
                        let mut win = window.write().unwrap();
                        win.set_decorated(true);
                        win.set_focus_on_show(false);
                        win.hide();
                    }

                    window.load_attributes();
                    {
                        let mut win = window.write().unwrap();
                        win.show();
                    }
                }
                _ => {}
            }
        }
    }
}
