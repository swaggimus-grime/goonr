use yew_router::Routable;

#[derive(Clone, Routable, PartialEq)]
enum Route {
    #[at("/api/upload_scene")]
    UploadScene,
    #[at("/api/get_scene")]
    GetScene,
}