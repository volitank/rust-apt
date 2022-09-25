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
		assert!(TagSection::new(
			"This-Is-Not-A-Valid-Control-File-Because-Its-Not-Colon-Separated"
		)
		.is_err());

		assert_eq!(control_section_one.get("Package").unwrap(), "pkg1");
		assert_eq!(control_section_one.get("Version").unwrap(), "1.0.0");
		assert_eq!(control_section_one.get("Description").unwrap(), "pkgdesc1");
		assert_eq!(
			control_section_one.get("Multi-Line").unwrap(),
			"Wow\n  This is\n  Multiple lines!"
		);
		assert_eq!(control_section_one.get("Back-To").unwrap(), "Normal");
		assert!(control_section_one
			.get("Not-A-Key-In-The-Control-File")
			.is_none());

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
}
