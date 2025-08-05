use yew::prelude::*;
use yew_router::prelude::*;

use crate::pages::{
    HomePage, 
    ModuleDocPage, 
    APIReferencePage, 
    TutorialPage, 
    SearchResultsPage
};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/modules/:module")]
    ModuleDoc { module: String },
    #[at("/api")]
    APIReference,
    #[at("/tutorials/:tutorial")]
    Tutorial { tutorial: String },
    #[at("/search")]
    SearchResults,
    #[not_found]
    #[at("/404")]
    NotFound,
}

pub fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <HomePage /> },
        Route::ModuleDoc { module } => html! { <ModuleDocPage module={module} /> },
        Route::APIReference => html! { <APIReferencePage /> },
        Route::Tutorial { tutorial } => html! { <TutorialPage tutorial={tutorial} /> },
        Route::SearchResults => html! { <SearchResultsPage /> },
        Route::NotFound => html! { <h1>{ "404 Not Found" }</h1> },
    }
}