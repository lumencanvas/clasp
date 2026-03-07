#include "ClaspCodec.h"

namespace ClaspCodec {

// Split-free wildcard matcher for CLASP address patterns.
// Patterns use '/' as segment separator.
// '*'  matches exactly one segment.
// '**' matches zero or more segments.
bool matchPattern(const char* pattern, const char* address) {
  // Both exhausted
  if (*pattern == '\0' && *address == '\0') return true;

  // '**' can match zero or more segments
  if (pattern[0] == '/' && pattern[1] == '*' && pattern[2] == '*') {
    // Skip the '/**'
    const char* rest = pattern + 3;

    // '/**' at end matches everything remaining
    if (*rest == '\0') return true;

    // Try matching rest against every suffix of address
    const char* a = address;
    // Try matching at current position (zero segments)
    if (matchPattern(rest, a)) return true;
    while (*a != '\0') {
      if (*a == '/') {
        if (matchPattern(rest, a)) return true;
      }
      a++;
    }
    return false;
  }

  // Leading '**' (at start of pattern)
  if (pattern[0] == '*' && pattern[1] == '*') {
    const char* rest = pattern + 2;
    if (*rest == '\0') return true;

    const char* a = address;
    if (matchPattern(rest, a)) return true;
    while (*a != '\0') {
      if (*a == '/') {
        if (matchPattern(rest, a)) return true;
      }
      a++;
    }
    return false;
  }

  // Both at a '/' separator -- advance both
  if (*pattern == '/' && *address == '/') {
    return matchPattern(pattern + 1, address + 1);
  }

  // '*' matches a single segment (everything up to next '/' or end)
  if (*pattern == '*') {
    const char* rest = pattern + 1;
    const char* a = address;
    // Consume the current segment in address
    while (*a != '\0' && *a != '/') a++;
    return matchPattern(rest, a);
  }

  // Literal character match
  if (*pattern != '\0' && *address != '\0' && *pattern == *address) {
    return matchPattern(pattern + 1, address + 1);
  }

  return false;
}

} // namespace ClaspCodec
