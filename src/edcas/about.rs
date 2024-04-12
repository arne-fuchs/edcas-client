use eframe::egui::ColorImage;
//TODO remove egui dependency
pub struct About {
    pub(crate) github_url: String,
    pub(crate) logo: ColorImage,
}

impl Default for About {
    fn default() -> Self {
        let mut logo_path = image::io::Reader::open("graphics\\logo\\edcas.png");
        if cfg!(target_os = "linux") {
            match image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas.png") {
                Ok(_) => {
                    logo_path =
                        image::io::Reader::open("/usr/share/edcas-client/graphics/logo/edcas.png");
                }
                Err(_) => {
                    logo_path = image::io::Reader::open("graphics/logo/edcas.png");
                }
            }
        }
        let image = logo_path.unwrap().decode().unwrap();
        let size = [image.width() as _, image.height() as _];
        let image_buffer = image.to_rgba8();
        let pixels = image_buffer.as_flat_samples();
        let color_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());
        Self {
            github_url: "https://github.com/arne-fuchs".to_owned(),
            logo: color_image,
        }
    }
}
