use crate::image::Image;

pub struct ImageFactory;

impl ImageFactory {
    pub fn create_images() -> Vec<Image> {
        let mut images = Vec::new();

        images.push(Image {
            vendor: "archlinux".to_string(),
            codename: "latest".to_string(),
            version: "latest".to_string(),
            url: "https://geo.mirror.pkgbuild.com/images/latest/Arch-Linux-x86_64-cloudimg.qcow2"
                .to_string(),
        });

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

        let fedora_url = "https://download.fedoraproject.org/pub/fedora/linux/releases/";
        images.push(Image {
            vendor: "fedora".to_string(),
            codename: "39".to_string(),
            version: "39".to_string(),
            url: fedora_url.to_string()
                + "39/Cloud/x86_64/images/Fedora-Cloud-Base-39-1.5.x86_64.qcow2",
        });
        images.push(Image {
            vendor: "fedora".to_string(),
            codename: "40".to_string(),
            version: "40".to_string(),
            url: fedora_url.to_string()
                + "40/Cloud/x86_64/images/Fedora-Cloud-Base-Generic.x86_64-40-1.14.qcow2",
        });
        images.push(Image{
            vendor: "fedora".to_string(),
            codename: "41".to_string(),
            version: "41".to_string(),
            url: fedora_url.to_string() + "test/41_Beta/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-41_Beta-1.2.x86_64.qcow2"
        });
        images.push(Image {
            vendor: "fedora".to_string(),
            codename: "42".to_string(),
            version: "42".to_string(),
            url: fedora_url.to_string()
                + "42/Cloud/x86_64/images/Fedora-Cloud-Base-Generic-42-1.1.x86_64.qcow2",
        });

        let ubuntu_releases = &[
            "bionic", "cosmic", "disco", "eoan", "focal", "groovy", "hirsute", "impish", "jammy",
            "kinetic", "lunar", "mantic", "noble", "oracular", "plucky",
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
