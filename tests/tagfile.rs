mod tagfile {
	use rust_apt::tagfile::{self, TagSection};

	#[test]
	fn correct() {
		let control_file = include_str!("files/tagfile/correct.control");
		let dpkg_status = include_str!("/var/lib/dpkg/status");
		let control_sections: Vec<&str> = control_file.split("\n\n").collect();
		let control_section_one = TagSection::new(control_sections.first().unwrap()).unwrap();
		let control_section_two = TagSection::new(control_sections.get(1).unwrap()).unwrap();

		assert!(tagfile::parse_tagfile(dpkg_status).is_ok());
		assert!(tagfile::parse_tagfile(control_file).is_ok());
		assert!(TagSection::new(control_file).is_err());
		assert!(
			TagSection::new("This-Is-Not-A-Valid-Control-File-Because-Its-Not-Colon-Separated")
				.is_err()
		);

		assert_eq!(control_section_one.get("Package").unwrap(), "pkg1");
		assert_eq!(control_section_one.get("Version").unwrap(), "1.0.0");
		assert_eq!(control_section_one.get("Description").unwrap(), "pkgdesc1");
		assert_eq!(
			control_section_one.get("Multi-Line").unwrap(),
			"Wow\n  This is\n  Multiple lines!"
		);
		assert_eq!(control_section_one.get("Back-To").unwrap(), "Normal");
		assert!(
			control_section_one
				.get("Not-A-Key-In-The-Control-File")
				.is_none()
		);

		assert_eq!(control_section_two.get("Package").unwrap(), "pkg2");
		assert_eq!(control_section_two.get("Version").unwrap(), "2.0.0");
		assert_eq!(control_section_two.get("Description").unwrap(), "pkgdesc2");
		assert_eq!(
			control_section_two.get("Value-Starts-On-Newline").unwrap(),
			"\n  Well that's interesting!\n  It's nice that this isn't failing the test, isn't \
			 it??"
		);
		assert_eq!(
			control_section_two.get("Normal-Line").unwrap(),
			"Once again"
		);
		assert_eq!(
			control_section_two.get("Tabbed-Indentation").unwrap(),
			"\n\tAll my homies know that tabs be superior.\n\t   Why not just use both?"
		);
	}

	#[test]
	fn malformed_later_section_returns_its_file_line() {
		let err = tagfile::parse_tagfile("Package: first\nVersion: 1\n\nPackage: second\nBroken")
			.unwrap_err();

		assert_eq!(err.line, Some(5));
	}

	#[test]
	fn repeated_separators_are_ignored() {
		let sections = tagfile::parse_tagfile("Package: first\n\n\n\nPackage: second\n").unwrap();

		assert_eq!(sections.len(), 2);
		assert_eq!(sections[0].get("Package").unwrap(), "first");
		assert_eq!(sections[1].get("Package").unwrap(), "second");
	}

	#[test]
	fn crlf_sections_are_supported() {
		let sections =
			tagfile::parse_tagfile("Package: first\r\nVersion: 1\r\n\r\nPackage: second\r\n")
				.unwrap();

		assert_eq!(sections.len(), 2);
		assert_eq!(sections[0].get("Version").unwrap(), "1");
		assert_eq!(sections[1].get("Package").unwrap(), "second");
	}
}
