// Copyright 2012-2014 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use ast::Name;
use std::cmp;
use parse::token::InternedString;

/// To find the Levenshtein distance between two strings
pub fn lev_distance(a: &str, b: &str) -> usize {
    // cases which don't require further computation
    if a.is_empty() {
        return b.chars().count();
    } else if b.is_empty() {
        return a.chars().count();
    }

    let mut dcol: Vec<_> = (0..b.len() + 1).collect();
    let mut t_last = 0;

    for (i, sc) in a.chars().enumerate() {
        let mut current = i;
        dcol[0] = current + 1;

        for (j, tc) in b.chars().enumerate() {
            let next = dcol[j + 1];
            if sc == tc {
                dcol[j + 1] = current;
            } else {
                dcol[j + 1] = cmp::min(current, next);
                dcol[j + 1] = cmp::min(dcol[j + 1], dcol[j]) + 1;
            }
            current = next;
            t_last = j;
        }
    } dcol[t_last + 1]
}

/// To find the best match for a given string from an iterator of names
/// As a loose rule to avoid the obviously incorrect suggestions, it takes
/// an optional limit for the maximum allowable edit distance, which defaults
/// to one-third of the given word
pub fn find_best_match_for_name<'a, T>(iter_names: T,
                                       lookup: &str,
                                       dist: Option<usize>) -> Option<InternedString>
    where T: Iterator<Item = &'a Name> {
    let max_dist = dist.map_or_else(|| cmp::max(lookup.len(), 3) / 3, |d| d);
    let it = iter_names
    .filter_map(|name| {
        let dist = lev_distance(lookup, &name.as_str());
        match dist <= max_dist {    // filter the unwanted cases
            true => Some((name.as_str(), dist)),
            false => None,
        }
    });
    min_by_key(it, |&(_, val)| val) // extract the tuple containing the minimum edit distance
    .map(|(s, _)| s)                // and return only the string
}

fn min_by_key<I, B: Ord, F>(it: I, f: F) -> Option<I::Item>
    where I: Iterator,
          F: FnMut(&I::Item) -> B,
{
    select_fold1(it,
                 f,
                 // only switch to y if it is strictly smaller, to
                 // preserve stability.
                 |x_p, _, y_p, _| x_p > y_p)
        .map(|(_, x)| x)
}

/// Select an element from an iterator based on the given projection
/// and "comparison" function.
///
/// This is an idiosyncratic helper to try to factor out the
/// commonalities of {max,min}{,_by}. In particular, this avoids
/// having to implement optimizations several times.
#[inline]
fn select_fold1<I,B, FProj, FCmp>(mut it: I,
                                  mut f_proj: FProj,
                                  mut f_cmp: FCmp) -> Option<(B, I::Item)>
    where I: Iterator,
          FProj: FnMut(&I::Item) -> B,
          FCmp: FnMut(&B, &I::Item, &B, &I::Item) -> bool
{
    // start with the first element as our selection. This avoids
    // having to use `Option`s inside the loop, translating to a
    // sizeable performance gain (6x in one case).
    it.next().map(|mut sel| {
        let mut sel_p = f_proj(&sel);

        for x in it {
            let x_p = f_proj(&x);
            if f_cmp(&sel_p,  &sel, &x_p, &x) {
                sel = x;
                sel_p = x_p;
            }
        }
        (sel_p, sel)
    })
}

#[test]
fn test_lev_distance() {
    use std::char::{from_u32, MAX};
    // Test bytelength agnosticity
    for c in (0..MAX as u32)
             .filter_map(|i| from_u32(i))
             .map(|i| i.to_string()) {
        assert_eq!(lev_distance(&c[..], &c[..]), 0);
    }

    let a = "\nMäry häd ä little lämb\n\nLittle lämb\n";
    let b = "\nMary häd ä little lämb\n\nLittle lämb\n";
    let c = "Mary häd ä little lämb\n\nLittle lämb\n";
    assert_eq!(lev_distance(a, b), 1);
    assert_eq!(lev_distance(b, a), 1);
    assert_eq!(lev_distance(a, c), 2);
    assert_eq!(lev_distance(c, a), 2);
    assert_eq!(lev_distance(b, c), 1);
    assert_eq!(lev_distance(c, b), 1);
}
