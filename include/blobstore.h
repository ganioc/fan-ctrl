#pragma once
#include "rust/cxx.h"
#include <cstdint>
#include <memory>

namespace org {
namespace blobstore {


class AdcClient {
public:
  AdcClient();
  uint16_t read(uint8_t channel) const;

private:
  class impl;
  std::shared_ptr<impl> impl;
};

std::unique_ptr<AdcClient> new_blobstore_client();

} // namespace blobstore
} // namespace org
