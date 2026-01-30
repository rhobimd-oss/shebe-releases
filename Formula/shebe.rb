class Shebe < Formula
  desc "BM25 full-text code search for AI agents via MCP"
  homepage "https://github.com/rhobimd-oss/shebe"
  version "0.5.7"
  license "MIT"

  on_macos do
    on_arm do
      url "https://github.com/rhobimd-oss/shebe/releases/download/" \
          "v#{version}/shebe-v#{version}-darwin-aarch64.tar.gz"
      sha256 "4f67b5cf6a090b61c5f357d3f79a4cec42e03ac7d15c57646dede2d79f1ed161"
    end

    on_intel do
      url "https://github.com/rhobimd-oss/shebe/releases/download/" \
          "v#{version}/shebe-v#{version}-darwin-x86_64.tar.gz"
      sha256 "a06b3b798f064f0adff99f3d0b987f7a9dff4adb09e5bf636e4e29bef15da8b1"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/rhobimd-oss/shebe/releases/download/" \
          "v#{version}/shebe-v#{version}-linux-x86_64.tar.gz"
      sha256 "cc353ef006be562e6464fdb15f1ed9eb7376209a93b869399593b532a0dab71b"
    end
  end

  def install
    bin.install "shebe"
    bin.install "shebe-mcp"
  end

  test do
    assert_match version.to_s,
                 shell_output("#{bin}/shebe --version")
  end
end
