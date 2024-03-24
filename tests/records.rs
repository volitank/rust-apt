mod records {
	use rust_apt::new_cache;
	use rust_apt::records::RecordField;

	#[test]
	fn fields() {
		let cache = new_cache!().unwrap();
		let pkg = cache.get("apt").unwrap();
		// let pkg = cache.get("nala").unwrap();
		let cand = pkg.candidate().unwrap();

		assert_eq!(
			cand.get_record(RecordField::Maintainer).unwrap(),
			"APT Development Team <deity@lists.debian.org>"
		);
		// Apt should not have a homepage
		assert!(cand.get_record(RecordField::Homepage).is_none());

		// This should also equal the same as the cand version
		assert_eq!(
			cand.get_record(RecordField::Version).unwrap(),
			cand.version()
		);

		// We can just print these for good luck.
		println!("Depends {:?}", cand.get_record(RecordField::Depends));
		println!("PreDepends {:?}", cand.get_record(RecordField::PreDepends));

		// This should be the same as what the Hash accessors will give.
		assert_eq!(cand.get_record("SHA256"), cand.sha256());
	}
}
