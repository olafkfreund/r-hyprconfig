# Multi-stage Dockerfile for r-hyprconfig

# Build stage with Nix
FROM nixos/nix:latest as builder

# Enable experimental features
RUN echo "experimental-features = nix-command flakes" >> /etc/nix/nix.conf

# Copy source code
WORKDIR /src
COPY . .

# Build the application with Nix
RUN nix build .#r-hyprconfig --no-interaction --accept-flake-config

# Runtime stage
FROM alpine:latest as runtime

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    gcompat \
    libgcc

# Create a non-root user
RUN addgroup -g 1000 app && \
    adduser -D -s /bin/sh -u 1000 -G app app

# Copy the binary from builder
COPY --from=builder /src/result/bin/r-hyprconfig /usr/local/bin/r-hyprconfig

# Set ownership and permissions
RUN chown app:app /usr/local/bin/r-hyprconfig && \
    chmod +x /usr/local/bin/r-hyprconfig

# Switch to non-root user
USER app
WORKDIR /home/app

# Set environment variables
ENV RUST_LOG=info
ENV TERM=xterm-256color

# Health check (optional)
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD r-hyprconfig --help || exit 1

# Default command
ENTRYPOINT ["/usr/local/bin/r-hyprconfig"]
CMD ["--help"]

# Development stage (for dev containers)
FROM builder as development

# Install development tools
RUN nix profile install nixpkgs#git nixpkgs#vim nixpkgs#curl

# Create development user
RUN addgroup -g 1000 dev && \
    adduser -D -s /bin/sh -u 1000 -G dev dev

# Copy source and set ownership
COPY --chown=dev:dev . /workspace
WORKDIR /workspace

# Switch to dev user
USER dev

# Set up development environment
ENV RUST_LOG=debug
ENV RUST_BACKTRACE=1

# Development entry point
ENTRYPOINT ["nix", "develop"]