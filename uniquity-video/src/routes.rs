use super::{
    handlers,
    keys::{
        EditedVideoDeleteModalKey, EditedVideoSelectTableKey, EditedVideoTableKey,
        PublishedVideoDeleteModalKey, PublishedVideoSelectTableKey, PublishedVideoTableKey,
        RawFootageDeleteModalKey, RawFootageSelectTableKey, RawFootageTableKey,
        VideoEmployeeSelectTableKey,
    },
};

lariv_rs::define_plugin_routes! {
    plugin: UniquityVideoTag;
    routes: [
        get VideoHubRouteTag, "/video/", handlers::hub::hub;
        get RawListRouteTag, "/video/raw/", handlers::raw::list, fragment(RawFootageTableKey);
        get RawCreateGetRouteTag, "/video/raw/create/", handlers::raw::create_get, modal;
        post RawCreatePostRouteTag, "/video/raw/create/", handlers::raw::create_post;
        get RawSelectRouteTag, "/video/raw/select/", handlers::raw::select, fragment(RawFootageSelectTableKey);
        get RawEmployeeSelectRouteTag, "/video/raw/select-employee/", handlers::raw::employee_select, fragment(VideoEmployeeSelectTableKey);
        get RawDetailRouteTag, "/video/raw/r/{id}/", handlers::raw::detail;
        get RawEditGetRouteTag, "/video/raw/r/{id}/edit/", handlers::raw::edit_get;
        post RawEditPostRouteTag, "/video/raw/r/{id}/edit/", handlers::raw::edit_post;
        get RawDeleteGetRouteTag, "/video/raw/r/{id}/delete/", handlers::raw::delete_get, modal;
        post RawDeletePostRouteTag, "/video/raw/r/{id}/delete/", bare handlers::raw::delete_post, fragment(RawFootageDeleteModalKey);
        get EditedListRouteTag, "/video/edited/", handlers::edited::list, fragment(EditedVideoTableKey);
        get EditedCreateGetRouteTag, "/video/edited/create/", handlers::edited::create_get, modal;
        post EditedCreatePostRouteTag, "/video/edited/create/", handlers::edited::create_post;
        get EditedSelectRouteTag, "/video/edited/select/", handlers::edited::select, fragment(EditedVideoSelectTableKey);
        get EditedDetailRouteTag, "/video/edited/r/{id}/", handlers::edited::detail;
        get EditedEditGetRouteTag, "/video/edited/r/{id}/edit/", handlers::edited::edit_get;
        post EditedEditPostRouteTag, "/video/edited/r/{id}/edit/", handlers::edited::edit_post;
        get EditedDeleteGetRouteTag, "/video/edited/r/{id}/delete/", handlers::edited::delete_get, modal;
        post EditedDeletePostRouteTag, "/video/edited/r/{id}/delete/", bare handlers::edited::delete_post, fragment(EditedVideoDeleteModalKey);
        get PublishedListRouteTag, "/video/published/", handlers::published::list, fragment(PublishedVideoTableKey);
        get PublishedCreateGetRouteTag, "/video/published/create/", handlers::published::create_get, modal;
        post PublishedCreatePostRouteTag, "/video/published/create/", handlers::published::create_post;
        get PublishedSelectRouteTag, "/video/published/select/", handlers::published::select, fragment(PublishedVideoSelectTableKey);
        get PublishedDetailRouteTag, "/video/published/r/{id}/", handlers::published::detail;
        get PublishedEditorPointsGetRouteTag, "/video/published/r/{id}/editor-points/", handlers::published::editor_points_get;
        post PublishedEditorPointsPostRouteTag, "/video/published/r/{id}/editor-points/", handlers::published::editor_points_post;
        get PublishedEditGetRouteTag, "/video/published/r/{id}/edit/", handlers::published::edit_get;
        post PublishedEditPostRouteTag, "/video/published/r/{id}/edit/", handlers::published::edit_post;
        get PublishedDeleteGetRouteTag, "/video/published/r/{id}/delete/", handlers::published::delete_get, modal;
        post PublishedDeletePostRouteTag, "/video/published/r/{id}/delete/", bare handlers::published::delete_post, fragment(PublishedVideoDeleteModalKey);
    ]
}
