#include "ruff-hnt-rs/include/adc.h"
#include "ruff-hnt-rs/src/main.rs.h"

#include <algorithm>
#include <cstdint>
#include <cstdio>
#include <functional>
#include <set>
#include <string>
#include <unordered_map>

#include <iostream>

#include "ruff-hnt-rs/include/ADS1X15_TLA2024.h"
namespace ruff {
namespace adc {

// Toy implementation of an in-memory adc.
//
// In reality the implementation of AdcClient could be a large complex C++
// library.
class AdcClient::impl {
  friend AdcClient;

  TLA2024 adc;
};

AdcClient::AdcClient() : impl(new class AdcClient::impl) {}

// Upload a new blob and return a blobid that serves as a handle to the blob.
uint16_t AdcClient::read(std::uint8_t channel) const {
  return impl->adc.readADC_SingleEnded(channel);
}


std::unique_ptr<AdcClient> new_adc_client() {
  return std::make_unique<AdcClient>();
}

} // namespace adc
} // namespace ruff
