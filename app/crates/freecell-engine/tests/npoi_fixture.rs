//! Regression test: a server-exported `.xlsx` whose `<styleSheet>` omits the optional
//! `<cellStyles>` element must open.
//!
//! `npoi_import_result.xlsx` is a real report produced by the TimeSheet/RMS import-result export
//! (NPOI 2.7.4, `docProps/app.xml` records `Application=NPOI`). Its `xl/styles.xml` declares only
//! `numFmts`, `fonts`, `fills`, `borders`, `cellStyleXfs` and `cellXfs` — `<cellStyles>`, `<dxfs>`,
//! `<tableStyles>` and `<colors>` are absent, all of which ECMA-376 marks optional.
//!
//! IronCalc's `load_styles` collected `<cellStyles>` and indexed `[0]`, so the empty node vec
//! panicked with an out-of-bounds index. The worker's `catch_unwind` turned that into
//! `LoadError::EnginePanic` — the "calculation engine crashed while opening this file" dialog —
//! even though the file is perfectly valid and contains no formulas at all.
//!
//! The fix lives in the pinned IronCalc fork (`styles.rs`: read every optional style collection
//! via `first()` and seed the index-0 default when absent). This test pins the user-visible
//! outcome: the workbook opens and its cells read back.

use std::path::PathBuf;

use freecell_core::CellRef;
use freecell_engine::WorkbookDocument;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/npoi_import_result.xlsx")
}

fn fmt(doc: &WorkbookDocument, row: u32, col: u32) -> String {
    doc.formatted_value(0, CellRef::new(row, col))
        .expect("cell in range")
}

#[test]
fn npoi_export_without_cell_styles_opens() {
    let doc = WorkbookDocument::open(&fixture()).expect(
        "an NPOI-exported .xlsx whose styleSheet omits the optional <cellStyles> must open",
    );

    assert_eq!(doc.sheet_count(), 1);
    assert_eq!(doc.sheet_names(), vec!["导入结果".to_string()]);

    // Header row A1:U1 — the export's column titles.
    assert_eq!(fmt(&doc, 0, 0), "导入结果");
    assert_eq!(fmt(&doc, 0, 1), "错误信息");
    assert_eq!(fmt(&doc, 0, 2), "员工工号");

    // Data rows: two successful imports (row 2 and 3), then a failure carrying the
    // quote-match error in B4.
    assert_eq!(fmt(&doc, 1, 0), "导入成功");
    assert_eq!(fmt(&doc, 1, 2), "611692");
    assert_eq!(fmt(&doc, 2, 0), "导入成功");
    assert_eq!(fmt(&doc, 2, 2), "611869");

    assert_eq!(fmt(&doc, 3, 0), "导入失败");
    assert_eq!(fmt(&doc, 3, 2), "377054");
    assert_eq!(
        fmt(&doc, 3, 1),
        "[报价匹配失败]未在项目关联销售合同匹配到唯一的报价信息"
    );
}
