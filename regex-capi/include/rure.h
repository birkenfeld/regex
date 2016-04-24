// Copyright 2014-2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#ifndef _RURE_H
#define _RURE_H

#include <stdbool.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct rure rure;

typedef struct rure_options rure_options;

typedef struct rure_error rure_error;

rure_error *rure_error_new();

void rure_error_free(rure_error *err);

const char *rure_error_message(rure_error *err);

typedef struct rure_match {
    size_t start;
    size_t end;
} rure_match;

typedef struct rure_captures rure_captures;

rure_captures *rure_captures_new(rure *re);

void rure_captures_free(rure_captures *captures);

bool rure_captures_at(rure_captures *captures, size_t i, rure_match *match);

size_t rure_captures_len(rure_captures *captures);

rure *rure_compile(const char *pattern, rure_error *error);

rure *rure_compile_must(const char *pattern);

rure *rure_compile_options(const uint8_t *pattern, size_t length,
                           rure_options *options, rure_error *error);

void rure_free(rure *re);

int32_t rure_capture_name_index(rure *re, const char *name);

bool rure_is_match(rure *re, const uint8_t *haystack, size_t length,
                   size_t start);

bool rure_find(rure *re, const uint8_t *haystack, size_t length,
               size_t start, rure_match *match);

bool rure_find_captures(rure *re, const uint8_t *haystack, size_t length,
                        size_t start, rure_captures *captures);

typedef struct rure_iter rure_iter;

rure_iter *rure_iter_new(rure *re, const uint8_t *haystack, size_t length);

void rure_iter_free(rure_iter *it);

bool rure_iter_next(rure_iter *it, rure_match *match);

bool rure_iter_next_captures(rure_iter *it, rure_captures *captures);

#ifdef __cplusplus
}
#endif

#endif
