//! # Design
//!
//!
//! # [`GRanges<R, T>`] Generic Types
//!
//! [`GRanges<R, T>`] types are generic over:
//!
//! 1. Their **range container** (`R`). This is because different operations to be fast, and thus
//!    need different data structures. The [`GRanges`] methods allow for efficient conversion of
//!    one range type to another.
//!
//! 2. The *optional* **data container** (`T`). The data container exists if the ranges have some
//!    associated data. When range *do* have data, there is an index that associates each range
//!    with its data element in the data container.
//!
//! This brings up the most important thing to know about working with the GRanges library: because
//! of the emphasis on on knowing all types at compile-time (which has performance benefits over
//! runtime type polymorphism), it must be known at compile-time whether ranges have data or not.
//!
//! In most applications this is known: a user specifies, for example, a GTF file and the contents
//! are parsed and processed accordingly. However, it's not unfeasible to imagine that one would
//! need runtime "polymorphism" over BED3 input (which would lead to a [`GRanges`] object without
//! data) and BED* (e.g. BED5, BED12, etc) input. (For example, the GRanges command line tool
//! `granges` runs into this problem — see it's implementation for examples.)
//!
//! These two possibilities are handled with two differently typed parsing iterators:
//! [`Bed3Iterator`] and [`BedlikeIterator`]. These yield different parsed range types,
//! [`GenomicRangeEmptyRecord`] and [`GenomicRangeRecord`], respectively.
//!
//!
//!
//! This is an important concept when working with [`GRanges<R, T>`] types:
//!
//! **High-level data types**: A key feature of GRanges design is that a single type of ranges are
//! contained in the range containers. By knowing that every range in a range container either has
//! an index to a data element in the data container or it does not ahead of time simplifies
//! downstream ergonomics tremendously.
//!
//! **Emphasis on compile-time**: For this, let's consider a common problem: a bioinformatics tool
//! needs to read in a BED-like file that has a variable, unknown at compile time, number of
//! columns.
//!
//! In Rust, this could be handled in one of two ways. First, it could be handled at *runtime*, by
//! leveraging Rust's dynamic trait system. For example, imagine loading in one of two possible BED
//! formats:
//!
//!  1. *Data-less BED3*: The appropriate `GRanges` object here would be a `GRanges<VecRangesEmpty,
//!     ()>`.
//!
//!  2. *BED-like with data*: Here, we'd need a `GRanges<VecRangesIndexed, Vec<U>>`, where the
//!     `Vec<U>` is data container containing just-loaded-in data.
//!
//! Suppose your code doesn't know, when you're writing it, which of these two cases it will
//! encounter.
//!
//! Because at compile-time, the types *need* to be known, there are a few options here.
//!

use std::path::PathBuf;

use genomap::GenomeMap;
use indexmap::IndexMap;

use crate::{
    io::{OutputFile},
    iterators::GRangesIterator,
    prelude::GRangesError,
    ranges::{
        coitrees::{COITrees, COITreesEmpty, COITreesIndexed},
        vec::{VecRanges, VecRangesEmpty, VecRangesIndexed},
        GenomicRangeEmptyRecord, GenomicRangeRecord, RangeEmpty, RangeIndexed,
    },
    traits::{
        GenericRange, GenomicRangesTsvSerialize,
        IndexedDataContainer, RangeContainer, IterableRangeContainer, TsvSerialize, AsGRangesRef, AdjustableGenericRange, 
    },
    Position, PositionOffset,
};

#[derive(Clone, Debug)]
pub struct GRanges<C, T> {
    pub(crate) ranges: GenomeMap<C>,
    pub(crate) data: Option<T>,
}

#[derive(Clone, Debug)]
pub struct GRangesEmpty<C>(GRanges<C, ()>);

impl<C> GRangesEmpty<C>
where
    C: RangeContainer,
{
    /// Get the total number of ranges.
    pub fn len(&self) -> usize {
        self.0.ranges.values().map(|ranges| ranges.len()).sum()
    }

    /// Return whether the [`GRanges`] object is empty (contains no ranges).
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    /// Get the raw range container.
    pub fn get_ranges(&self, seqname: &str) -> Option<&C> {
        self.0.ranges.get(seqname)
    }

    /// Get the sequence names.
    pub fn seqnames(&self) -> Vec<String> {
        self.0.ranges.names()
    }

    /// Get the sequences lengths.
    pub fn seqlens(&self) -> IndexMap<String, Position> {
        let seqlens = self.0
            .ranges
            .iter()
            .map(|(seqname, ranges)| (seqname.to_string(), ranges.sequence_length()))
            .collect();
        seqlens
    }
}

impl<C> From<GRangesEmpty<C>> for GRanges<C, ()> {
    fn from(value: GRangesEmpty<C>) -> Self {
        value.0
    }
}

impl<'a, C> AsGRangesRef<'a, C, ()> for GRangesEmpty<C> {
    /// Convert a reference to a [`GRangesEmpty<C>`] to a reference to the 
    /// underlying [`GRanges<C, ()>`]. This is to greatly improve the ergonomics
    /// of functions that could take either a [`GRanges`] or [`GRangesEmpty] type.
    fn as_granges_ref(&'a self) -> &'a GRanges<C, ()> {
        &self.0
    }
}

impl<'a, C, T> AsGRangesRef<'a, C, T> for GRanges<C, T> {
    /// Return a reference of a [`GRanges<C, T>`] object. This is essentially
    /// a pass-through method. [`IntoGRangesRef`] is not needed in this case,
    /// but is needed elsewhere (see the implementation for [`GRangesEmpty`]) to 
    /// improve the ergonomics of working with [`GRanges`] and [`GRangesEmpty`] types.
    fn as_granges_ref(&'a self) -> &'a GRanges<C, T> {
        self
    }
}



impl<C, T> GRanges<C, T>
where
    C: RangeContainer,
{
    /// Get the total number of ranges.
    pub fn len(&self) -> usize {
        self.ranges.values().map(|ranges| ranges.len()).sum()
    }

    /// Return whether the [`GRanges`] object is empty (contains no ranges).
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Get the raw range container.
    pub fn get_ranges(&self, seqname: &str) -> Option<&C> {
        self.ranges.get(seqname)
    }

    /// Get the sequence names.
    pub fn seqnames(&self) -> Vec<String> {
        self.ranges.names()
    }

    /// Get the sequences lengths.
    pub fn seqlens(&self) -> IndexMap<String, Position> {
        let seqlens = self
            .ranges
            .iter()
            .map(|(seqname, ranges)| (seqname.to_string(), ranges.sequence_length()))
            .collect();
        seqlens
    }
}

impl<'a, T> GenomicRangesTsvSerialize<'a, VecRangesIndexed> for GRanges<VecRangesIndexed, T>
where
    T: IndexedDataContainer<'a>,
    T: TsvSerialize,
    <T as IndexedDataContainer<'a>>::Item: TsvSerialize,
{
    /// Write
    fn to_tsv(&'a self, output: Option<impl Into<PathBuf>>) -> Result<(), GRangesError> {
        // output stream -- header is None for now (TODO)
        let output = output.map_or(OutputFile::new_stdout(None), |file| {
            OutputFile::new(file, None)
        });
        let mut writer = output.writer()?;

        let seqnames = self.seqnames();
        for range in self.iter_ranges() {
            let record = range.to_record(&seqnames, self.data.as_ref());
            writeln!(writer, "{}", record.to_tsv())?;
        }
        Ok(())
    }
}

impl<'a, R: IterableRangeContainer> GenomicRangesTsvSerialize<'a, R> for GRangesEmpty<R> {
    /// Output a BED3 file for for this data-less [`GRanges<R, ()>`].
    fn to_tsv(&'a self, output: Option<impl Into<PathBuf>>) -> Result<(), GRangesError> {
        // output stream -- header is None for now (TODO)
        let output = output.map_or(OutputFile::new_stdout(None), |file| {
            OutputFile::new(file, None)
        });
        let mut writer = output.writer()?;

        let seqnames = self.seqnames();
        for range in self.0.iter_ranges() {
            let record = range.to_record_empty::<()>(&seqnames);
            writeln!(writer, "{}", record.to_tsv())?;
        }
        Ok(())
    }
}

impl<R: GenericRange, T> GRanges<VecRanges<R>, T> {
    /// Create a new [`GRanges`] object, with vector storage for ranges and data.
    ///
    /// This combination of range and data containers is used when loading data into
    /// a new [`GRanges`] object, and the size cannot be known beforehand. Rust's
    /// [`Vec`] will dynamically grow to accommodate new ranges; use [`GRanges.shrink()`]
    /// call the [`Vec`]'s shrink to size methods on the range and data containers
    /// after data loading to shrink to the minimal necessary size (this can reduce
    /// memory usage).
    pub fn new_vec(seqlens: &IndexMap<String, Position>) -> Self {
        let mut ranges = GenomeMap::new();
        for (seqname, length) in seqlens.iter() {
            // this should never happen because the error is only if
            // insert encounters a seqname that's already been inserted -- that
            // cannot happen here.
            ranges
                .insert(seqname, VecRanges::new(*length))
                .expect("Internal error: please report");
        }
        Self { ranges, data: None }
    }

    /// Consume this [`GRanges`] object and sort the ranges.
    pub fn sort(mut self) -> Self {
        self.ranges.values_mut().for_each(|ranges| ranges.sort());
        self
    }
   
    pub fn shink(&mut self) {
        todo!()
    }
}


impl<R: AdjustableGenericRange, T> GRanges<VecRanges<R>, T> {
    /// Adjust all the ranges in this [`GRanges`] object in place.
    pub fn adjust_ranges(mut self, start_delta: PositionOffset, end_delta: PositionOffset) -> Self {
        self.ranges
            .values_mut()
            .for_each(|ranges| ranges.adjust_ranges(start_delta, end_delta));
        self
    }
}
 

impl<R: GenericRange> GRangesEmpty<VecRanges<R>> {
    /// Create a new [`GRangesEmpty`] object, with vector storage for ranges and no 
    /// data container.
    pub fn new_vec(seqlens: &IndexMap<String, Position>) -> Self {
        GRangesEmpty(GRanges::new_vec(seqlens))
    }

    pub fn sort(self) -> Self {
        GRangesEmpty(self.0.sort())
    }

    pub fn shink(&mut self) {
        todo!()
    }
} 

impl<R: AdjustableGenericRange> GRangesEmpty<VecRanges<R>> {
    pub fn adjust_ranges(self, start_delta: PositionOffset, end_delta: PositionOffset) -> Self {
        GRangesEmpty(self.0.adjust_ranges(start_delta, end_delta))
    }
}

impl<U> GRanges<VecRangesIndexed, Vec<U>> {
    /// Push a genomic range with its data to the range and data containers in a [`GRanges] object.
    pub fn push_range(
        &mut self,
        seqname: &str,
        start: Position,
        end: Position,
        data: U,
    ) -> Result<(), GRangesError> {
        // push data to the vec data container, getting the index
        let index: usize = {
            let data_container = self.data.get_or_insert_with(Vec::new);
            data_container.push(data);
            data_container.len() - 1 // new data index
        };
        // push an indexed range
        let range = RangeIndexed::new(start, end, index);
        let range_container = self
            .ranges
            .get_mut(seqname)
            .ok_or(GRangesError::MissingSequence(seqname.to_string()))?;
        range_container.push_range(range);
        Ok(())
    }
}

impl<'a, T> GRanges<VecRanges<RangeIndexed>, T>
where
    T: IndexedDataContainer<'a>,
    T: TsvSerialize,
    <T as IndexedDataContainer<'a>>::Item: TsvSerialize,
{
    ///
    pub fn to_tsv(&'a self, output: Option<impl Into<PathBuf>>) -> Result<(), GRangesError> {
        // output stream -- header is None for now (TODO)
        let output = output.map_or(OutputFile::new_stdout(None), |file| {
            OutputFile::new(file, None)
        });
        let mut writer = output.writer()?;

        let seqnames = self.seqnames();
        for range in self.iter_ranges() {
            let record = range.to_record(&seqnames, self.data.as_ref());
            writeln!(writer, "{}", record.to_tsv())?;
        }
        Ok(())
    }
}

impl<'a, C, T> GRanges<C, T>
where
    T: IndexedDataContainer<'a>,
{
    /// Get the data in the data container at specified index.
    ///
    /// # Panics
    /// This will panic if there if the index is invalid, or the
    /// data container is `None`. Both of these indicate internal
    /// design errors: please file an issue of you encounter a panic.
    pub fn get_data_value(&'a self, index: usize) -> <T as IndexedDataContainer>::Item {
        self.data
            .as_ref()
            .expect("data container was None")
            .get_value(index)
    }
}

impl GRangesEmpty<VecRanges<RangeEmpty>> {
    /// Push an empty range (no data) to the appropriate [`VecRangesEmpty`] range container.
    pub fn push_range(
        &mut self,
        seqname: &str,
        start: Position,
        end: Position,
    ) -> Result<(), GRangesError> {
        // push an unindexed (empty) range
        let range = RangeEmpty::new(start, end);
        let range_container = self.0
            .ranges
            .get_mut(seqname)
            .ok_or(GRangesError::MissingSequence(seqname.to_string()))?;
        range_container.push_range(range);
        Ok(())
    }
}

impl<T> GRanges<VecRanges<RangeIndexed>, T> {
    /// Push an empty range (no data) to the [`VecRangesEmpty`] range container.
    pub fn push_range_with_index(
        &mut self,
        seqname: &str,
        start: Position,
        end: Position,
        index: usize,
    ) -> Result<(), GRangesError> {
        // push an unindexed (empty) range
        let range = RangeIndexed::new(start, end, index);
        let range_container = self
            .ranges
            .get_mut(seqname)
            .ok_or(GRangesError::MissingSequence(seqname.to_string()))?;
        range_container.push_range(range);
        Ok(())
    }
}

impl<U> GRanges<VecRangesIndexed, Vec<U>> {
    /// Create a new [`GRanges<VecRangesIndexed, Vec<U>>`] object from an iterator over
    /// [`GenomicRangeRecord<U>`] records.
    pub fn from_iter<I>(
        iter: I,
        seqlens: &IndexMap<String, Position>,
    ) -> Result<GRanges<VecRangesIndexed, Vec<U>>, GRangesError>
    where
        I: Iterator<Item = Result<GenomicRangeRecord<U>, GRangesError>>,
    {
        let mut gr = GRanges::new_vec(seqlens);
        for possible_entry in iter {
            let entry = possible_entry?;
            gr.push_range(&entry.seqname, entry.start, entry.end, entry.data)?;
        }
        Ok(gr)
    }
}

impl GRangesEmpty<VecRangesEmpty> {
    /// Create a new [`GRanges<VecRangesEmpty, Vec<U>>`] object from an iterator over
    /// [`GenomicRangeRecord`] records.
    pub fn from_iter<I>(
        iter: I,
        seqlens: &IndexMap<String, Position>,
    ) -> Result<GRangesEmpty<VecRangesEmpty>, GRangesError>
    where
        I: Iterator<Item = Result<GenomicRangeEmptyRecord, GRangesError>>,
    {
        let mut gr = GRangesEmpty::new_vec(seqlens);
        for possible_entry in iter {
            let entry = possible_entry?;
            gr.push_range(&entry.seqname, entry.start, entry.end)?;
        }
        Ok(gr)
    }
}


impl<C> GRangesEmpty<C> 
where COITrees<()>: From<C> {
    /// Convert the [`VecRangesEmpty`] range containers in this [`GRangesEmpty`] to a
    /// cache-oblivious interval tree range container, [`COITreesEmpty`]. This is 
    /// done using the [`coitrees`] library by Daniel C. Jones.
    pub fn to_coitrees(self) -> Result<GRangesEmpty<COITreesEmpty>, GRangesError> {
        let old_ranges = self.0.ranges;
        let mut new_ranges = GenomeMap::new();
        for (seqname, vec_ranges) in old_ranges.into_iter() {
            let trees = COITrees::from(vec_ranges);
            new_ranges.insert(&seqname, trees)?;
        }
        Ok(GRangesEmpty(GRanges {
            ranges: new_ranges,
            data: None,
        }))
    }
}

impl<T> GRanges<VecRanges<RangeIndexed>, T> {
    /// Convert the [`VecRangesIndexed`] range containers in this [`GRanges`] to a 
    /// cache-oblivious interval tree range container, [`COITreesIndexed`]. This is 
    /// done using the [`coitrees`] library by Daniel C. Jones.
    pub fn to_coitrees(self) -> Result<GRanges<COITreesIndexed, T>, GRangesError> {
        let old_ranges = self.ranges;
        let mut new_ranges = GenomeMap::new();
        for (seqname, vec_ranges) in old_ranges.into_iter() {
            let trees = COITrees::from(vec_ranges);
            new_ranges.insert(&seqname, trees)?;
        }
        Ok(GRanges {
            ranges: new_ranges,
            data: self.data,
        })
    }
}

impl<CL: RangeContainer> GRangesEmpty<CL>
where
    CL: IterableRangeContainer,
{
    /// Filter out ranges that do *not* have at least overlap with the `right` ranges.
    ///
    /// In database lingo, this is a type of *filtering join*, in particular a *semi join*.
    /// See Hadley Wickham's excellent [R for Data
    /// Science](https://r4ds.hadley.nz/joins.html#filtering-joins) for more information.
    ///
    /// Note that this consumes the `self` [`GRanges`] object, turning it into a new
    /// [`GRanges<VecRangesEmpty, ()>`].
    ///
    pub fn filter_overlaps<'a, M: Clone + 'a, DR: 'a>(
        self,
        // right: &GRanges<COITrees<M>, DR>,
        right: &'a impl AsGRangesRef<'a, COITrees<M>, DR>
    ) -> Result<GRangesEmpty<VecRangesEmpty>, GRangesError> {
        let mut gr = GRangesEmpty::new_vec(&self.seqlens());

        let right_ref = right.as_granges_ref();

        for (seqname, left_ranges) in self.0.ranges.iter() {
            for left_range in left_ranges.iter_ranges() {
                if let Some(right_ranges) = right_ref.ranges.get(seqname) {
                    let num_overlaps =
                        right_ranges.count_overlaps(left_range.start(), left_range.end());
                    if num_overlaps == 0 {
                        // no overlaps -- skip
                    } else {
                        gr.push_range(seqname, left_range.start(), left_range.end())?;
                    }
                }
            }
        }
        Ok(gr)
    }
}

impl<CL, U> GRanges<CL, Vec<U>>
where
    CL: IterableRangeContainer,
{
    //pub fn left_overlaps<DR>(self, right: &'a GRanges<COITreesIndexed, DR>)
    //-> GRanges<CL, JoinIterator<'a, CL, Vec<U>, DR>> {
    //    //let mut obj = GRanges {
    //    //    ranges: self.ranges,
    //    //    data: None,
    //    //};
    //    //obj.data = Some(JoinIterator::new(&obj, &right));
    //    //obj
    //    todo!()
    //}

    /// Filter out ranges that do *not* have at least overlap with the `right` ranges.
    ///
    /// In database lingo, this is a type of *filtering join*, in particular a *semi join*.
    /// See Hadley Wickham's excellent [R for Data
    /// Science](https://r4ds.hadley.nz/joins.html#filtering-joins) for more information.
    ///
    /// Note that this consumes the `self` [`GRanges`] object, turning it into a new
    /// [`GRanges<VecRangesIndexed, Vec<U>`]. The data container is rebuilt from indices
    /// into a new [`Vec<U>`] where `U` is the associated type [`IndexedDataContainer::Item`],
    /// which represents the individual data element in the data container.
    pub fn filter_overlaps<DR>(
        self,
        right: &GRanges<COITreesIndexed, DR>,
    ) -> Result<GRanges<VecRangesIndexed, Vec<U>>, GRangesError> {
        let mut gr: GRanges<VecRangesIndexed, Vec<U>> = GRanges::new_vec(&self.seqlens());

        for (seqname, left_ranges) in self.ranges.iter() {
            for left_range in left_ranges.iter_ranges() {
                if let Some(right_ranges) = right.ranges.get(seqname) {
                    let num_overlaps =
                        right_ranges.count_overlaps(left_range.start(), left_range.end());
                    if num_overlaps == 0 {
                        // no overlaps -- skip
                    } else {
                        gr.push_range_with_index(
                            seqname,
                            left_range.start(),
                            left_range.end(),
                            left_range.index().unwrap(),
                        )?;
                    }
                }
            }
        }
        Ok(gr)
    }
}



impl<R, T> GRanges<R, T>
where
R: IterableRangeContainer,
{
    /// Create a new [`GRangesIterator`] to iterate through all the ranges in this [`GRanges`] object.
    pub fn iter_ranges(&self) -> GRangesIterator<'_, R> {
        GRangesIterator::new(&self.ranges)
    }
}


impl<R> GRangesEmpty<R>
where
R: IterableRangeContainer,
{
    /// Create a new [`GRangesIterator`] to iterate through all the ranges in this [`GRangesEmpty`] object.
    pub fn iter_ranges(&self) -> GRangesIterator<'_, R> {
        GRangesIterator::new(&self.0.ranges)
    }
}


#[cfg(test)]
mod tests {
    use indexmap::indexmap;

    use crate::{
        prelude::*,
        test_utilities::{granges_test_case_01, random_vecranges},
    };

    #[test]
    fn test_new_vec() {
        let seqlens = indexmap! { "chr1".to_string() => 10};
        let mut gr = GRanges::new_vec(&seqlens);
        gr.push_range("chr1", 0, 10, 1.1).unwrap();
        assert_eq!(gr.len(), 1);
    }

    #[test]
    fn test_random_vecranges() {
        let vr = random_vecranges(100);
        assert_eq!(vr.len(), 100)
    }

    #[test]
    fn test_to_coitrees() {
        let gr_vec = granges_test_case_01();
        let gr = gr_vec.clone().to_coitrees().unwrap();
        assert_eq!(gr.len(), 5);
    }
}
