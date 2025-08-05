use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod routes;
mod services;

use routes::{Route, switch};

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Switch<Route> render={switch} />
        </BrowserRouter>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}

// Feature flags for documentation system
#[cfg(feature = "search")]
mod search {
    // Advanced search implementation
}

#[cfg(feature = "interactive")]
mod interactive {
    // Code playground and live examples
}

#[cfg(test)]
mod tests {
    // Documentation system tests
}