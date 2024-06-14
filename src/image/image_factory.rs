use crate::image::Image;

pub struct ImageFactory;

impl ImageFactory {
    pub fn create_images() -> Vec<Image> {
        let mut images = Vec::new();

        let debian_releases = &[
            "stretch", "buster", "bullseye", "bookworm", "trixie", "forky",
        ];
        let debian_url = "https://cloud.debian.org/images/cloud/{name}/latest/debian-{version}-generic-amd64.qcow2";
        let vendor = "debian";
        for (counter, codename) in debian_releases.iter().enumerate() {
            let version = counter + 9;
            images.push(Image {
                vendor: vendor.to_string(),
                codename: codename.to_string(),
                version: version.to_string(),
                url: debian_url
                    .replace("{name}", codename)
                    .replace("{version}", &version.to_string()),
            });
        }

        let ubuntu_releases = &[
            "bionic", "cosmic", "disco", "eoan", "focal", "groovy", "hirsute", "impish", "jammy",
            "kinetic", "lunar", "mantic", "noble", "oracular",
        ];
        let ubuntu_url = "https://cloud-images.ubuntu.com/minimal/releases/{name}/release/ubuntu-{version}-minimal-cloudimg-amd64.img";
        let vendor = "ubuntu";
        for (counter, codename) in ubuntu_releases.iter().enumerate() {
            let year = 18 + (counter / 2);
            let month = if counter % 2 == 0 { 4 } else { 10 };
            let version = format!("{year}.{month:02}");
            images.push(Image {
                vendor: vendor.to_string(),
                codename: codename.to_string(),
                version: version.clone(),
                url: ubuntu_url
                    .replace("{name}", codename)
                    .replace("{version}", &version.clone()),
            });
        }

        images
    }
}
