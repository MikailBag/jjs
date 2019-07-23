#include <cstdint>
#include "util.h"


static const uintptr_t SIGN_EXTENSION = 0xffff'0000'0000'0000;

bool check_pointer(void* ptr) {
    const auto p = (uintptr_t) ptr;
    const auto sign_ext = p & SIGN_EXTENSION;
    return (sign_ext == SIGN_EXTENSION) || (sign_ext == 0);
}