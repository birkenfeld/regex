#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

#include "rure.h"

#ifndef DEBUG
  #define DEBUG false
#endif

bool test_is_match() {
    bool passed = true;
    const char *haystack = "snowman: \xE2\x98\x83";

    rure *re = rure_compile_must("(?u)\\p{So}$");
    bool matched = rure_is_match(re, (const uint8_t *)haystack,
                                 strlen(haystack), 0);
    if (!matched) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_is_match] expected match, "
                    "but got no match\n");
        }
        passed = false;
    }
    rure_free(re);
    return passed;
}

bool test_find() {
    bool passed = true;
    const char *haystack = "snowman: \xE2\x98\x83";

    rure *re = rure_compile_must("(?u)\\p{So}$");
    rure_match match = {0};
    bool matched = rure_find(re, (const uint8_t *)haystack, strlen(haystack),
                             0, &match);
    if (!matched) {
        if (DEBUG) {
            fprintf(stderr, "[test_find] expected match, but got no match\n");
        }
        passed = false;
    }
    size_t expect_start = 9;
    size_t expect_end = 12;
    if (match.start != expect_start || match.end != expect_end) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_find] expected match at (%zu, %zu), but "
                    "got match at (%zu, %zu)\n",
                    expect_start, expect_end, match.start, match.end);
        }
        passed = false;
    }
    rure_free(re);
    return passed;
}

bool test_captures() {
    bool passed = true;
    const char *haystack = "snowman: \xE2\x98\x83";

    rure *re = rure_compile_must("(?u).(.*(?P<snowman>\\p{So}))$");
    rure_match match = {0};
    rure_captures *caps = rure_captures_new(re);
    bool matched = rure_find_captures(re, (const uint8_t *)haystack,
                                      strlen(haystack), 0, caps);
    if (!matched) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_captures] expected match, but got no match\n");
        }
        passed = false;
    }
    int32_t expect_capture_index = 2;
    int32_t capture_index = rure_capture_name_index(re, "snowman");
    if (capture_index != expect_capture_index) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_captures] "
                    "expected capture index %d for name 'snowman', but "
                    "got %d\n",
                    expect_capture_index, capture_index);
        }
        passed = false;
        goto done;
    }
    size_t expect_start = 9;
    size_t expect_end = 12;
    rure_captures_at(caps, 2, &match);
    if (match.start != expect_start || match.end != expect_end) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_captures] "
                    "expected capture 2 match at (%zu, %zu), "
                    "but got match at (%zu, %zu)\n",
                    expect_start, expect_end, match.start, match.end);
        }
        passed = false;
    }
done:
    rure_captures_free(caps);
    rure_free(re);
    return passed;
}

bool test_iter() {
    bool passed = true;
    const char *haystack = "abc xyz";

    rure *re = rure_compile_must("\\w+(\\w)");
    rure_match match = {0};
    rure_iter *it = rure_iter_new(re, (const uint8_t *)haystack,
                                  strlen(haystack));

    bool matched = rure_iter_next(it, &match);
    if (!matched) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_iter] expected first match, but got no match\n");
        }
        passed = false;
        goto done;
    }
    size_t expect_start = 0;
    size_t expect_end = 3;
    if (match.start != expect_start || match.end != expect_end) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_iter] expected first match at (%zu, %zu), but "
                    "got match at (%zu, %zu)\n",
                    expect_start, expect_end, match.start, match.end);
        }
        passed = false;
        goto done;
    }

    rure_captures *caps = rure_captures_new(re);
    matched = rure_iter_next_captures(it, caps);
    if (!matched) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_iter] expected second match, but got no match\n");
        }
        passed = false;
        goto done;
    }
    rure_captures_at(caps, 1, &match);
    expect_start = 6;
    expect_end = 7;
    if (match.start != expect_start || match.end != expect_end) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_iter] expected second match at (%zu, %zu), but "
                    "got match at (%zu, %zu)\n",
                    expect_start, expect_end, match.start, match.end);
        }
        passed = false;
        goto done;
    }
done:
    rure_iter_free(it);
    rure_free(re);
    return passed;
}

bool test_compile_error() {
    bool passed = true;
    rure_error *err = rure_error_new();
    rure *re = rure_compile("(", err);
    if (re != NULL) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_compile_error] "
                    "expected NULL regex pointer, but got non-NULL pointer\n");
        }
        passed = false;
        rure_free(re);
    }
    const char *msg = rure_error_message(err);
    if (NULL == strstr(msg, "Unclosed parenthesis")) {
        if (DEBUG) {
            fprintf(stderr,
                    "[test_compile_error] "
                    "expected an 'unclosed parenthesis' error message, but "
                    "got this instead: '%s'\n", msg);
        }
        passed = false;
    }
    return passed;
}

int main() {
    bool passed = true;
    if (!test_is_match()) {
        passed = false;
        fprintf(stderr, "FAILED: test_is_match\n");
    }
    if (!test_find()) {
        passed = false;
        fprintf(stderr, "FAILED: test_find\n");
    }
    if (!test_captures()) {
        passed = false;
        fprintf(stderr, "FAILED: test_captures\n");
    }
    if (!test_iter()) {
        passed = false;
        fprintf(stderr, "FAILED: test_captures\n");
    }
    if (!test_compile_error()) {
        passed = false;
        fprintf(stderr, "FAILED: test_compile_error\n");
    }
    if (!passed) {
        exit(1);
    }
}
