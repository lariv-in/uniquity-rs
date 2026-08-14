use super::{
    handlers,
    keys::{
        GandolaSelectModalKey, GandolaSelectTableKey, GandolaTableKey, SiteSelectModalKey,
        SiteSelectTableKey, SiteTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: GandolaManagerTag;
    routes: [
        get GandolaDefaultRouteTag, "/gandola", handlers::gandolas::list, fragment(GandolaTableKey);
        get GandolaCreateGetRouteTag, "/gandola/create", handlers::gandolas::create_get, modal;
        post GandolaCreatePostRouteTag, "/gandola/create", handlers::gandolas::create_post;
        get GandolaDetailRouteTag, "/gandola/g/{id}", handlers::gandolas::detail;
        get GandolaEditGetRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_get, modal;
        post GandolaEditPostRouteTag, "/gandola/g/{id}/edit", handlers::gandolas::edit_post;
        post GandolaDeletePostRouteTag, "/gandola/g/{id}/delete", bare handlers::gandolas::delete_post, redirect;
        get GandolaSelectRouteTag, "/gandola/pick", handlers::gandolas::select, multi_select(GandolaSelectTableKey, GandolaSelectModalKey);

        get SiteDefaultRouteTag, "/gandola/sites", handlers::sites::list, fragment(SiteTableKey);
        get SiteCreateGetRouteTag, "/gandola/sites/create", handlers::sites::create_get, modal;
        post SiteCreatePostRouteTag, "/gandola/sites/create", handlers::sites::create_post;
        get SiteDetailRouteTag, "/gandola/sites/s/{id}", handlers::sites::detail;
        get SiteEditGetRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_get, modal;
        post SiteEditPostRouteTag, "/gandola/sites/s/{id}/edit", handlers::sites::edit_post;
        post SiteDeletePostRouteTag, "/gandola/sites/s/{id}/delete", bare handlers::sites::delete_post, redirect;
        get SiteSelectRouteTag, "/gandola/sites/pick", handlers::sites::select, multi_select(SiteSelectTableKey, SiteSelectModalKey);

        get GandolaPreferencesRouteTag, "/gandola/preferences", handlers::preferences::get;
        post GandolaPreferencesPostRouteTag, "/gandola/preferences", handlers::preferences::post;
    ]
}
