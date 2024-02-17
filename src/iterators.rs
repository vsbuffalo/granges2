use genomap::GenomeMap;

use crate::{
    ranges::RangeIndexedRecord,
    traits::{GenericRange, RangeContainer, RangesIterable},
};

/// An iterator yielding [`RangeIndexedRecord`], which store
/// indices to the sequence names and data container.
///
/// # Developer Notes
/// Using indices, rather than references to data items and `&str` directly,
/// prevents lifetime complexity.
pub struct GRangesIterator<'a, R>
where
    R: RangesIterable,
{
    ranges: &'a GenomeMap<R>,
    current_seqname_index: usize,
    current_range_iter: Box<dyn Iterator<Item = <R as RangesIterable>::RangeType> + 'a>,
}

impl<'a, R> GRangesIterator<'a, R>
where
    R: RangesIterable,
{
    pub fn new(ranges: &'a GenomeMap<R>) -> Self {
        let current_range_iter = ranges.get_by_index(0).unwrap().iter_ranges();
        Self {
            ranges,
            current_seqname_index: 0,
            current_range_iter,
        }
    }
}

impl<'a, R> Iterator for GRangesIterator<'a, R>
where
    R: RangeContainer + RangesIterable,
    <R as RangesIterable>::RangeType: GenericRange,
{
    type Item = RangeIndexedRecord;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(next_range) = self.current_range_iter.next() {
                return Some(RangeIndexedRecord {
                    seqname_index: self.current_seqname_index,
                    start: next_range.start(),
                    end: next_range.end(),
                    index: next_range.index(),
                });
            } else {
                // try to load another sequence's set of ranges.
                self.current_seqname_index += 1;
                if self.current_seqname_index >= self.ranges.len() {
                    // we're out of range container iterators
                    return None;
                }
                self.current_range_iter = self
                    .ranges
                    .get_by_index(self.current_seqname_index)
                    .unwrap()
                    .iter_ranges();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ranges::RangeIndexedRecord, test_utilities::granges_test_case_01};

    use super::GRangesIterator;

    #[test]
    fn test_genomic_ranges_iterator() {
        let gr = granges_test_case_01();
        let mut iter = GRangesIterator::new(&gr.ranges);
        assert_eq!(
            iter.next().unwrap(),
            RangeIndexedRecord::new(0, 0, 5, Some(0))
        );
    }
}
