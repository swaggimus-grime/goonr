#[function_component(UploadColmap)]
pub fn upload_colmap() -> Html {
    let file_name = use_state(|| None::<String>);
    let on_file_upload = {
        let file_name = file_name.clone();
        Callback::from(move |event: Event| {
            let input: web_sys::HtmlInputElement = event.target_unchecked_into();
            let files: FileList = input.files().unwrap();
            if let Some(file) = files.get(0) {
                let name = file.name();
                file_name.set(Some(name.clone()));

                let reader = FileReader::new().unwrap();
                let fr = GlooFileReader::read_as_bytes(&file).unwrap();

                wasm_bindgen_futures::spawn_local(async move {
                    let data = fr.await.unwrap();
                    // TODO: Pass `data` to your ZIP parser or WASM processor
                    log::info!("Loaded file with {} bytes", data.len());
                });
            }
        })
    };

    html! {
        <div>
            <input type="file" accept=".zip" onchange={on_file_upload} />
            if let Some(name) = (*file_name).clone() {
                <p>{ format!("Uploaded: {}", name) }</p>
            }
        </div>
    }
}