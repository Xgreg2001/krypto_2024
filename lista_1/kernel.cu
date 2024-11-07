#include <cstddef>
#include <cstdint>
#include <cstdio>
#include <curand_kernel.h>
#include <stdio.h>

// For the CUDA runtime routines (prefixed with "cuda_")
#include <cuda_runtime.h>
#include <sys/types.h>

#define STATE_SIZE_WORDS 4
#define CANDIDATES_SIZE_WORDS 16

#define A1_ONE_BITS 0x84200000u
#define A1_ZERO_BITS 0x0A000820u
#define D1_ONE_BITS 0x8C000800u
#define D1_ZERO_BITS 0x02208026u
#define D1_A1_SAME_BITS 0x701F10C0u
#define C1_ONE_BITS 0xBE1F0966u
#define C1_ZERO_BITS 0x40201080u
#define C1_D1_SAME_BITS 0x00000018u
#define B1_ONE_BITS 0xBA040010u
#define B1_ZERO_BITS 0x443B19EEu
#define B1_C1_SAME_BITS 0x00000601u
#define A2_ONE_BITS 0x482F0E50u
#define A2_ZERO_BITS 0xB41011AFu
#define D2_ONE_BITS 0x04220C56u
#define D2_ZERO_BITS 0x9A1113A9u
#define C2_ONE_BITS 0x96011E01u
#define C2_ZERO_BITS 0x083201C0u
#define C2_D2_SAME_BITS 0x01808000u
#define B2_ONE_BITS 0x843283C0u
#define B2_ZERO_BITS 0x1B810001u
#define B2_C2_SAME_BITS 0x00000002u
#define A3_ONE_BITS 0x9C0101C1u
#define A3_ZERO_BITS 0x03828202u
#define A3_B2_SAME_BITS 0x00001000u
#define D3_ONE_BITS 0x878383C0u
#define D3_ZERO_BITS 0x00041003u
#define C3_ONE_BITS 0x800583C3u
#define C3_ZERO_BITS 0x00021000u
#define C3_D3_SAME_BITS 0x00086000u
#define B3_ONE_BITS 0x80081080u
#define B3_ZERO_BITS 0x0007E000u
#define B3_C3_SAME_BITS 0x7F000000u
#define A4_ONE_BITS 0x3F0FE008u
#define A4_ZERO_BITS 0xC0000080u
#define D4_ONE_BITS 0x400BE088u
#define D4_ZERO_BITS 0xBF040000u
#define C4_ONE_BITS 0x7D000000u
#define C4_ZERO_BITS 0x82008008u
#define B4_ONE_BITS 0x20000000u
#define B4_ZERO_BITS 0x80000000u
#define A5_ZERO_BITS 0x80020000u
#define A5_B4_SAME_BITS 0x00008008u
#define D5_ONE_BITS 0x00020000u
#define D5_ZERO_BITS 0x80000000u
#define D5_A5_SAME_BITS 0x20000000u
#define C5_ZERO_BITS 0x80020000u
#define B5_ZERO_BITS 0x80000000u
#define A6_ZERO_BITS 0x80000000u
#define A6_B5_SAME_BITS 0x00020000u
#define D6_ZERO_BITS 0x80000000u
#define C6_ZERO_BITS 0x80000000u
#define B6_C6_DIFFERENT_BITS 0x80000000u
#define B12_D12_SAME_BITS 0x80000000u
#define A13_C12_SAME_BITS 0x80000000u
#define D13_B12_DIFFERENT_BITS 0x80000000u
#define C13_A13_SAME_BITS 0x80000000u
#define B13_D13_SAME_BITS 0x80000000u
#define A14_C13_SAME_BITS 0x80000000u
#define D14_B13_SAME_BITS 0x80000000u
#define C14_A14_SAME_BITS 0x80000000u
#define B14_D14_SAME_BITS 0x80000000u
#define A15_C14_SAME_BITS 0x80000000u
#define D15_B14_SAME_BITS 0x80000000u
#define C15_A15_SAME_BITS 0x80000000u
#define B15_D15_DIFFERENT_BITS 0x80000000u
#define A16_ONE_BITS 0x02000000u
#define A16_C15_SAME_BITS 0x80000000u
#define D16_ONE_BITS 0x02000000u
#define D16_B15_SAME_BITS 0x80000000u

__device__ uint32_t _rotl(const uint32_t value, uint32_t shift) {
  if ((shift &= sizeof(value) * 8 - 1) == 0)
    return value;
  return (value << shift) | (value >> (sizeof(value) * 8 - shift));
}

__device__ uint32_t _rotr(const uint32_t value, uint32_t shift) {
  if ((shift &= sizeof(value) * 8 - 1) == 0)
    return value;
  return (value >> shift) | (value << (sizeof(value) * 8 - shift));
}

__device__ uint32_t F(uint32_t x, uint32_t y, uint32_t z) {
  return (x & y) | ((~x) & z);
}

__device__ uint32_t G(uint32_t x, uint32_t y, uint32_t z) {
  return (x & z) | (y & ~z);
}

__device__ uint32_t H(uint32_t x, uint32_t y, uint32_t z) { return x ^ y ^ z; }

__device__ uint32_t I(uint32_t x, uint32_t y, uint32_t z) {
  return y ^ (x | ~z);
}

__device__ uint32_t FF(uint32_t a, uint32_t b, uint32_t c, uint32_t d,
                       uint32_t word, uint32_t k, uint32_t s) {
  uint32_t f = a + F(b, c, d) + word + k;
  return _rotl(f, s) + b;
}

__device__ uint32_t GG(uint32_t a, uint32_t b, uint32_t c, uint32_t d,
                       uint32_t word, uint32_t k, uint32_t s) {
  uint32_t g = a + G(b, c, d) + word + k;
  return _rotl(g, s) + b;
}

__device__ uint32_t HH(uint32_t a, uint32_t b, uint32_t c, uint32_t d,
                       uint32_t word, uint32_t k, uint32_t s) {
  uint32_t h = a + H(b, c, d) + word + k;
  return _rotl(h, s) + b;
}

__device__ uint32_t II(uint32_t a, uint32_t b, uint32_t c, uint32_t d,
                       uint32_t word, uint32_t k, uint32_t s) {
  uint32_t i = a + I(b, c, d) + word + k;
  return _rotl(i, s) + b;
}

__device__ uint32_t apply_one_bits(uint32_t v, uint32_t mask) {
  return v | mask;
}

__device__ uint32_t apply_zero_bits(uint32_t v, uint32_t mask) {
  return v & (~mask);
}

__device__ uint32_t apply_same_bits(uint32_t v, uint32_t u, uint32_t mask) {
  return (v | (u & mask)) & (u | (~mask));
}

__device__ bool verify_one_bits(uint32_t v, uint32_t mask) {
  return (v & mask) == mask;
}

__device__ bool verify_zero_bits(uint32_t v, uint32_t mask) {
  return (v & mask) == 0;
}

__device__ bool verify_same_bits(uint32_t v, uint32_t u, uint32_t mask) {
  return (v & mask) == (u & mask);
}

__device__ bool verify_different_bits(uint32_t v, uint32_t u, uint32_t mask) {
  return (v & mask) != (u & mask);
}

__device__ uint32_t reverse_FF(uint32_t a, uint32_t b, uint32_t c, uint32_t d,
                               uint32_t t, uint32_t s, uint32_t orig) {
  return _rotr((a - b), s) - F(b, c, d) - orig - t;
}

__global__ void generate_candidates(const uint32_t md5_state[STATE_SIZE_WORDS],
                                    uint32_t *candidates, bool *found,
                                    size_t iterations, size_t seed) {
  size_t idx = blockIdx.x * blockDim.x + threadIdx.x;

  curandState random_state;
  curand_init(seed, idx, 0, &random_state);

  uint32_t words[16];

  for (size_t i = 0; i < iterations; i++) {

    for (int j = 0; j < 16; ++j) {
      words[j] = curand(&random_state);
    }

    uint32_t a = md5_state[0];
    uint32_t b = md5_state[1];
    uint32_t c = md5_state[2];
    uint32_t d = md5_state[3];

    uint32_t orig;

    // ROUND 1

    //   a1
    orig = a;
    a = FF(a, b, c, d, words[0], 0xD76AA478u, 7u);
    a = apply_one_bits(a, A1_ONE_BITS);
    a = apply_zero_bits(a, A1_ZERO_BITS);
    words[0] = reverse_FF(a, b, c, d, 0xD76AA478, 7, orig);

    // d1
    orig = d;
    d = FF(d, a, b, c, words[1], 0xE8C7B756, 12);
    d = apply_one_bits(d, D1_ONE_BITS);
    d = apply_zero_bits(d, D1_ZERO_BITS);
    d = apply_same_bits(d, a, D1_A1_SAME_BITS);
    words[1] = reverse_FF(d, a, b, c, 0xE8C7B756, 12, orig);

    // c1
    orig = c;
    c = FF(c, d, a, b, words[2], 0x242070DB, 17);
    c = apply_one_bits(c, C1_ONE_BITS);
    c = apply_zero_bits(c, C1_ZERO_BITS);
    c = apply_same_bits(c, d, C1_D1_SAME_BITS);
    words[2] = reverse_FF(c, d, a, b, 0x242070DB, 17, orig);

    // b1
    orig = b;
    b = FF(b, c, d, a, words[3], 0xC1BDCEEE, 22);
    b = apply_one_bits(b, B1_ONE_BITS);
    b = apply_zero_bits(b, B1_ZERO_BITS);
    b = apply_same_bits(b, c, B1_C1_SAME_BITS);
    words[3] = reverse_FF(b, c, d, a, 0xC1BDCEEE, 22, orig);

    // a2
    orig = a;
    a = FF(a, b, c, d, words[4], 0xF57C0FAF, 7);
    a = apply_one_bits(a, A2_ONE_BITS);
    a = apply_zero_bits(a, A2_ZERO_BITS);
    words[4] = reverse_FF(a, b, c, d, 0xF57C0FAF, 7, orig);

    // d2
    orig = d;
    d = FF(d, a, b, c, words[5], 0x4787C62A, 12);
    d = apply_one_bits(d, D2_ONE_BITS);
    d = apply_zero_bits(d, D2_ZERO_BITS);
    words[5] = reverse_FF(d, a, b, c, 0x4787C62A, 12, orig);

    // c2
    orig = c;
    c = FF(c, d, a, b, words[6], 0xA8304613, 17);
    c = apply_one_bits(c, C2_ONE_BITS);
    c = apply_zero_bits(c, C2_ZERO_BITS);
    c = apply_same_bits(c, d, C2_D2_SAME_BITS);
    words[6] = reverse_FF(c, d, a, b, 0xA8304613, 17, orig);

    // b2
    orig = b;
    b = FF(b, c, d, a, words[7], 0xFD469501, 22);
    b = apply_one_bits(b, B2_ONE_BITS);
    b = apply_zero_bits(b, B2_ZERO_BITS);
    b = apply_same_bits(b, c, B2_C2_SAME_BITS);
    words[7] = reverse_FF(b, c, d, a, 0xFD469501, 22, orig);

    // a3
    orig = a;
    a = FF(a, b, c, d, words[8], 0x698098D8, 7);
    a = apply_one_bits(a, A3_ONE_BITS);
    a = apply_zero_bits(a, A3_ZERO_BITS);
    a = apply_same_bits(a, b, A3_B2_SAME_BITS);
    words[8] = reverse_FF(a, b, c, d, 0x698098D8, 7, orig);

    // d3
    orig = d;
    d = FF(d, a, b, c, words[9], 0x8B44F7AF, 12);
    d = apply_one_bits(d, D3_ONE_BITS);
    d = apply_zero_bits(d, D3_ZERO_BITS);
    words[9] = reverse_FF(d, a, b, c, 0x8B44F7AF, 12, orig);

    // c3
    orig = c;
    c = FF(c, d, a, b, words[10], 0xFFFF5BB1, 17);
    c = apply_one_bits(c, C3_ONE_BITS);
    c = apply_zero_bits(c, C3_ZERO_BITS);
    c = apply_same_bits(c, d, C3_D3_SAME_BITS);
    words[10] = reverse_FF(c, d, a, b, 0xFFFF5BB1, 17, orig);

    // b3
    orig = b;
    b = FF(b, c, d, a, words[11], 0x895CD7BE, 22);
    b = apply_one_bits(b, B3_ONE_BITS);
    b = apply_zero_bits(b, B3_ZERO_BITS);
    b = apply_same_bits(b, c, B3_C3_SAME_BITS);
    words[11] = reverse_FF(b, c, d, a, 0x895CD7BE, 22, orig);

    // a4
    orig = a;
    a = FF(a, b, c, d, words[12], 0x6B901122, 7);
    a = apply_one_bits(a, A4_ONE_BITS);
    a = apply_zero_bits(a, A4_ZERO_BITS);
    words[12] = reverse_FF(a, b, c, d, 0x6B901122, 7, orig);

    // d4
    orig = d;
    d = FF(d, a, b, c, words[13], 0xFD987193, 12);
    d = apply_one_bits(d, D4_ONE_BITS);
    d = apply_zero_bits(d, D4_ZERO_BITS);
    words[13] = reverse_FF(d, a, b, c, 0xFD987193, 12, orig);

    // c4
    orig = c;
    c = FF(c, d, a, b, words[14], 0xA679438E, 17);
    c = apply_one_bits(c, C4_ONE_BITS);
    c = apply_zero_bits(c, C4_ZERO_BITS);
    words[14] = reverse_FF(c, d, a, b, 0xA679438E, 17, orig);

    // b4
    orig = b;
    b = FF(b, c, d, a, words[15], 0x49B40821, 22);
    b = apply_one_bits(b, B4_ONE_BITS);
    b = apply_zero_bits(b, B4_ZERO_BITS);
    words[15] = reverse_FF(b, c, d, a, 0x49B40821, 22, orig);

    // ROUND 2

    // a5
    a = GG(a, b, c, d, words[1], 0xF61E2562, 5);
    if (!verify_zero_bits(a, A5_ZERO_BITS)) {
      continue;
    }
    if (!verify_same_bits(a, b, A5_B4_SAME_BITS)) {
      continue;
    }

    // d5
    d = GG(d, a, b, c, words[6], 0xC040B340, 9);
    if (!verify_zero_bits(d, D5_ZERO_BITS)) {
      continue;
    }
    if (!verify_one_bits(d, D5_ONE_BITS)) {
      continue;
    }
    if (!verify_same_bits(d, a, D5_A5_SAME_BITS)) {
      continue;
    }

    // c5
    c = GG(c, d, a, b, words[11], 0x265E5A51, 14);
    if (!verify_zero_bits(c, C5_ZERO_BITS)) {
      continue;
    }

    // b5
    b = GG(b, c, d, a, words[0], 0xE9B6C7AA, 20);
    if (!verify_zero_bits(b, B5_ZERO_BITS)) {
      continue;
    }

    // a6
    a = GG(a, b, c, d, words[5], 0xD62F105D, 5);
    if (!verify_zero_bits(a, A6_ZERO_BITS)) {
      continue;
    }
    if (!verify_same_bits(a, b, A6_B5_SAME_BITS)) {
      continue;
    }

    // d6
    d = GG(d, a, b, c, words[10], 0x02441453, 9);
    if (!verify_zero_bits(d, D6_ZERO_BITS)) {
      continue;
    }

    // c6
    c = GG(c, d, a, b, words[15], 0xD8A1E681, 14);
    if (!verify_zero_bits(c, C6_ZERO_BITS)) {
      continue;
    }

    // b6
    b = GG(b, c, d, a, words[4], 0xE7D3FBC8, 20);
    if (!verify_different_bits(b, c, B6_C6_DIFFERENT_BITS)) {
      continue;
    }

    a = GG(a, b, c, d, words[9], 0x21E1CDE6, 5);
    d = GG(d, a, b, c, words[14], 0xC33707D6, 9);
    c = GG(c, d, a, b, words[3], 0xF4D50D87, 14);
    b = GG(b, c, d, a, words[8], 0x455A14ED, 20);

    a = GG(a, b, c, d, words[13], 0xA9E3E905, 5);
    d = GG(d, a, b, c, words[2], 0xFCEFA3F8, 9);
    c = GG(c, d, a, b, words[7], 0x676F02D9, 14);
    b = GG(b, c, d, a, words[12], 0x8D2A4C8A, 20);

    // ROUND 3

    a = HH(a, b, c, d, words[5], 0xFFFA3942, 4);
    d = HH(d, a, b, c, words[8], 0x8771F681, 11);
    c = HH(c, d, a, b, words[11], 0x6D9D6122, 16);
    b = HH(b, c, d, a, words[14], 0xFDE5380C, 23);

    a = HH(a, b, c, d, words[1], 0xA4BEEA44, 4);
    d = HH(d, a, b, c, words[4], 0x4BDECFA9, 11);
    c = HH(c, d, a, b, words[7], 0xF6BB4B60, 16);
    b = HH(b, c, d, a, words[10], 0xBEBFBC70, 23);

    a = HH(a, b, c, d, words[13], 0x289B7EC6, 4);
    d = HH(d, a, b, c, words[0], 0xEAA127FA, 11);
    c = HH(c, d, a, b, words[3], 0xD4EF3085, 16);
    b = HH(b, c, d, a, words[6], 0x04881D05, 23);

    a = HH(a, b, c, d, words[9], 0xD9D4D039, 4);
    d = HH(d, a, b, c, words[12], 0xE6DB99E5, 11);
    c = HH(c, d, a, b, words[15], 0x1FA27CF8, 16);

    // b12
    b = HH(b, c, d, a, words[2], 0xC4AC5665, 23);
    if (!verify_same_bits(b, d, B12_D12_SAME_BITS)) {
      continue;
    }

    // ROUND 4

    // a13
    a = II(a, b, c, d, words[0], 0xF4292244, 6);
    if (!verify_same_bits(a, c, A13_C12_SAME_BITS)) {
      continue;
    }

    // d13
    d = II(d, a, b, c, words[7], 0x432AFF97, 10);
    if (!verify_different_bits(d, b, D13_B12_DIFFERENT_BITS)) {
      continue;
    }

    // c13
    c = II(c, d, a, b, words[14], 0xAB9423A7, 15);
    if (!verify_same_bits(c, a, C13_A13_SAME_BITS)) {
      continue;
    }

    // b13
    b = II(b, c, d, a, words[5], 0xFC93A039, 21);
    if (!verify_same_bits(b, d, B13_D13_SAME_BITS)) {
      continue;
    }

    // a14
    a = II(a, b, c, d, words[12], 0x655B59C3, 6);
    if (!verify_same_bits(a, c, A14_C13_SAME_BITS)) {
      continue;
    }

    // d14
    d = II(d, a, b, c, words[3], 0x8F0CCC92, 10);
    if (!verify_same_bits(d, b, D14_B13_SAME_BITS)) {
      continue;
    }

    // c14
    c = II(c, d, a, b, words[10], 0xFFEFF47D, 15);
    if (!verify_same_bits(c, a, C14_A14_SAME_BITS)) {
      continue;
    }

    // b14
    b = II(b, c, d, a, words[1], 0x85845DD1, 21);
    if (!verify_same_bits(b, d, B14_D14_SAME_BITS)) {
      continue;
    }

    // a15
    a = II(a, b, c, d, words[8], 0x6FA87E4F, 6);
    if (!verify_same_bits(a, c, A15_C14_SAME_BITS)) {
      continue;
    }

    // d15
    d = II(d, a, b, c, words[15], 0xFE2CE6E0, 10);
    if (!verify_same_bits(d, b, D15_B14_SAME_BITS)) {
      continue;
    }

    // c15
    c = II(c, d, a, b, words[6], 0xA3014314, 15);
    if (!verify_same_bits(c, a, C15_A15_SAME_BITS)) {
      continue;
    }

    // b15
    b = II(b, c, d, a, words[13], 0x4E0811A1, 21);
    if (!verify_different_bits(b, d, B15_D15_DIFFERENT_BITS)) {
      continue;
    }

    // a16
    a = II(a, b, c, d, words[4], 0xF7537E82, 6);
    if (!verify_one_bits(a, A16_ONE_BITS)) {
      continue;
    }
    if (!verify_same_bits(a, c, A16_C15_SAME_BITS)) {
      continue;
    }

    // d16
    d = II(d, a, b, c, words[11], 0xBD3AF235, 10);
    if (!verify_one_bits(d, D16_ONE_BITS)) {
      continue;
    }
    if (!verify_same_bits(d, b, D16_B15_SAME_BITS)) {
      continue;
    }

    for (size_t j = 0; j < 16; j++) {
      candidates[idx * 16 + j] = words[j];
    }

    found[idx] = true;

    break;
  }
}

/**
 * Host main routine
 */
extern "C" {

int generate_candidates_cuda(const uint32_t *state, uint32_t *candidates,
                             bool *found, const size_t iterations,
                             const size_t threadsPerBlock,
                             const size_t blockDim, const size_t seed) {
  // Error code to check return values for CUDA callsd
  cudaError_t err = cudaSuccess;

  size_t batch_size = threadsPerBlock * blockDim;

  uint32_t *d_state = NULL;
  err = cudaMalloc((void **)&d_state, sizeof(uint32_t) * STATE_SIZE_WORDS);

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to allocate device vector state (error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  uint32_t *d_candidates = NULL;
  err = cudaMalloc((void **)&d_candidates,
                   sizeof(uint32_t) * batch_size * CANDIDATES_SIZE_WORDS);

  if (err != cudaSuccess) {
    fprintf(stderr,
            "Failed to allocate device vector candidates (error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  bool *d_found = NULL;
  err = cudaMalloc((void **)&d_found, sizeof(bool) * batch_size);

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to allocate device vector found (error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaMemcpy(d_state, state, sizeof(uint32_t) * STATE_SIZE_WORDS,
                   cudaMemcpyHostToDevice);

  if (err != cudaSuccess) {
    fprintf(stderr,
            "Failed to copy vector state from host to device (error code "
            "%s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  generate_candidates<<<blockDim, threadsPerBlock>>>(d_state, d_candidates,
                                                     d_found, iterations, seed);
  err = cudaGetLastError();

  if (err != cudaSuccess) {
    fprintf(stderr,
            "Failed to launch validateCandidates kernel (error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaDeviceSynchronize();

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to synchronize the device! error=%s\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaMemcpy(candidates, d_candidates, sizeof(uint32_t) * batch_size * 16,
                   cudaMemcpyDeviceToHost);

  if (err != cudaSuccess) {
    fprintf(stderr,
            "Failed to copy vector candidates from device to host (error code "
            "%s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaMemcpy(found, d_found, sizeof(uint8_t) * batch_size,
                   cudaMemcpyDeviceToHost);

  if (err != cudaSuccess) {
    fprintf(stderr,
            "Failed to copy vector found from device to host (error code "
            "%s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaFree(d_candidates);

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to free device vector candidates(error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaFree(d_found);

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to free device vector found (error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaFree(d_state);

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to free device vector state (error code %s)!\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  err = cudaDeviceReset();

  if (err != cudaSuccess) {
    fprintf(stderr, "Failed to deinitialize the device! error=%s\n",
            cudaGetErrorString(err));
    exit(EXIT_FAILURE);
  }

  return 0;
}
}
