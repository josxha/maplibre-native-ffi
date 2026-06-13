#pragma once

#include <cstring>
#include <iosfwd>
#include <string>

// NDK libc++ (LLVM 19+) no longer provides std::char_traits for unsigned short.
// JavaCPP's JNI helpers still use that type for UTF-16 jchar arrays.
namespace std {
template <>
struct char_traits<unsigned short> {
  using char_type = unsigned short;
  using int_type = int;
  using off_type = streamoff;
  using pos_type = streampos;
  using state_type = mbstate_t;

  static void assign(char_type& r, const char_type& a) noexcept { r = a; }
  static constexpr bool eq(char_type a, char_type b) noexcept { return a == b; }
  static constexpr bool lt(char_type a, char_type b) noexcept { return a < b; }
  static char_type* assign(char_type* p, size_t count, char_type a) {
    for (size_t i = 0; i < count; ++i) {
      p[i] = a;
    }
    return p;
  }
  static size_t length(const char_type* s) {
    size_t i = 0;
    while (!eq(s[i], char_type())) {
      ++i;
    }
    return i;
  }
  static int compare(const char_type* s1, const char_type* s2, size_t count) {
    for (size_t i = 0; i < count; ++i) {
      if (lt(s1[i], s2[i])) {
        return -1;
      }
      if (lt(s2[i], s1[i])) {
        return 1;
      }
    }
    return 0;
  }
  static char_type* copy(char_type* dest, const char_type* src, size_t count) {
    return static_cast<char_type*>(
      memcpy(dest, src, count * sizeof(char_type))
    );
  }
  static char_type* move(char_type* dest, const char_type* src, size_t count) {
    return static_cast<char_type*>(
      memmove(dest, src, count * sizeof(char_type))
    );
  }
  static const char_type* find(
    const char_type* s, size_t count, const char_type& ch
  ) {
    for (size_t i = 0; i < count; ++i) {
      if (eq(s[i], ch)) {
        return s + i;
      }
    }
    return nullptr;
  }
  static char_type to_char_type(int_type c) noexcept {
    return static_cast<char_type>(c);
  }
  static int_type to_int_type(char_type c) noexcept {
    return static_cast<int_type>(c);
  }
  static bool eq_int_type(int_type a, int_type b) noexcept { return a == b; }
  static int_type eof() noexcept { return EOF; }
  static int_type not_eof(int_type c) noexcept { return c == eof() ? 0 : c; }
};
}  // namespace std
