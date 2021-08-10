#pragma once
#include "rust/cxx.h"
#include <cstdint>
#include <memory>

namespace ruff {
namespace adc {


class AdcClient {
public:
  AdcClient();
  uint16_t read(uint8_t channel) const;

private:
  class impl;
  std::shared_ptr<impl> impl;
};

std::unique_ptr<AdcClient> new_adc_client();

} // namespace adc
} // namespace ruff
