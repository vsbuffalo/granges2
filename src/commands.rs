use std::path::PathBuf;

use crate::{
    io::OutputFile,
    prelude::*,
    ranges::operations::adjust_range,
    reporting::{CommandOutput, Report},
    test_utilities::random_granges,
    traits::{IterableRangeContainer, TsvSerialize},
    PositionOffset,
};

/// Adjust the genomic ranges in a bedfile by some specified amount.
// NOTE: we don't do build the full GRanges objects here, for efficiency.
// But it would be a good benchmark to see how much slower that is.
pub fn granges_adjust(
    bedfile: &PathBuf,
    seqlens: &PathBuf,
    both: PositionOffset,
    output: Option<&PathBuf>,
    sort: bool,
) -> Result<CommandOutput<()>, GRangesError> {
    let genome = read_seqlens(seqlens)?;

    // create the parsing iterator, and detect which variant we need based on
    // column number of the first entry.
    let bedlike_iterator = BedlikeIterator::new(bedfile)?;

    // output stream -- header is None for now (TODO)
    let output_stream = output.map_or(OutputFile::new_stdout(None), |file| {
        OutputFile::new(file, None)
    });
    let mut writer = output_stream.writer()?;

    // for reporting stuff to the user
    let mut report = Report::new();

    let mut skipped_ranges = 0;

    if !sort {
        // if we don't need to sort, use iterator-based streaming processing
        for record in bedlike_iterator {
            let range = record?;
            let seqname = &range.seqname;
            let length = *genome
                .get(seqname)
                .ok_or(GRangesError::MissingSequence(seqname.to_string()))?;

            let possibly_adjusted_range = adjust_range(range, -both, both, length);

            if let Some(range_adjusted) = possibly_adjusted_range {
                writer.write_all(&range_adjusted.to_tsv().into_bytes())?;
            } else {
                skipped_ranges += 1;
            }

            if skipped_ranges > 0 {
                report.add_issue(format!(
                    "{} ranges were removed because their widths after adjustment were ≤ 0",
                    skipped_ranges
                ))
            }
        }
    } else {
        // if we do need to sort, build up a GRanges variant and adjust ranges that way
        // let mut gr = GRanges::from_iter(bedlike_iterator, &genome)?;
        // gr.adjust_ranges(-both, both).to_tsv(output)?
    }

    Ok(CommandOutput::new((), report))
}
//
// /// Retain only the ranges that have at least one overlap with another set of ranges.
// pub fn granges_filter<DL, DR>(
//     seqlens: &PathBuf,
//     left_granges: GRanges<VecRangesEmpty, DL>,
//     right_granges: GRanges<VecRangesEmpty, DR>,
//     sort: bool,
// ) -> Result<CommandOutput<()>, GRangesError>
// where
//     // we must be able to iterate over left ranges
//     VecRangesEmpty: IterableRangeContainer, 
//     // we must be able to convert the right GRanges to interval trees
//      GRanges<VecRangesEmpty, ()>: GenomicRangesToIntervalTrees<()>,
// {
//     let right_granges = right_granges.to_coitrees()?;
//
//     // let intersection = left_granges.filter_overlaps_(&right_granges)?;
//
//     //// output stream -- header is None for now (TODO)
//     //let output_stream = output.map_or(OutputFile::new_stdout(None), |file| {
//     //    OutputFile::new(file, None)
//     //});
//     //let mut writer = output_stream.writer()?;
//
//     // for reporting stuff to the user
//     let mut report = Report::new();
//
//     //intersection.sort().to_tsv(output)?;
//
//     Ok(CommandOutput::new((), report))
// }

/// Generate a random BED-like file with genomic ranges.
pub fn granges_random_bed(
    seqlens: impl Into<PathBuf>,
    num: u32,
    output: Option<impl Into<PathBuf>>,
    sort: bool,
) -> Result<CommandOutput<()>, GRangesError> {
    // get the genome info
    let genome = read_seqlens(seqlens)?;

    let mut gr = random_granges(&genome, num.try_into().unwrap())?;

    if sort {
        gr = gr.sort();
    }

    gr.to_tsv(output)?;

    let report = Report::new();
    Ok(CommandOutput::new((), report))
}
