use std::rc::Rc;
use winit::{self, dpi::LogicalSize};

#[cfg(target_arch = "wasm32")]
trait WinitWeb {
    /// Add a canvas element to the HTML body and enable resize support.
    fn init_web(self: &Rc<Self>);

    /// Get the size of the browser client area.
    fn get_client_size() -> LogicalSize<f64>;
}

#[cfg(target_arch = "wasm32")]
impl WinitWeb for winit::window::Window {
    fn init_web(self: &Rc<Self>) {
        use wasm_bindgen::JsCast;
        use winit::platform::web::WindowExtWebSys;

        // Attach winit canvas to body element
        web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.body())
            .and_then(|body| {
                body.append_child(&web_sys::Element::from(self.canvas()))
                    .ok()
            })
            .expect("couldn't append canvas to document body");
        
        // Initialize winit window with current dimensions of browser client
        self.set_inner_size(Self::get_client_size());

        let winit_window = Rc::clone(&self);
        let closure = wasm_bindgen::closure::Closure::wrap(Box::new(move |_e: web_sys::Event| {
            winit_window.set_inner_size(Self::get_client_size());
        }) as Box<dyn FnMut(_)>);

        // Listen for resize event on browser client. Adjust winit window dimensions on event trigger
        let client_window = web_sys::window().unwrap();
        client_window
            .add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())
            .unwrap();
        
        closure.forget();
    }

    fn get_client_size() -> LogicalSize<f64> {
        let client_window = web_sys::window().unwrap();
        LogicalSize::new(
            client_window.inner_width().unwrap().as_f64().unwrap(),
            client_window.inner_height().unwrap().as_f64().unwrap()
        )
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
