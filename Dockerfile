# We use the latest Rust stable release as base image
FROM rust:1.59.0 AS builder

# Let's switch our working directory to `app` (equivalent to `cd app`)
# The `app` folder will be created for us by Docker in case it does not
# exist already.
WORKDIR /app

# Install the required system dependencies for our linking configuration
RUN apt update && apt install lld clang -y

# Copy all files from our working environment to our Docker image
COPY . .

# SQLX_OFFLINE environment variable to true in our Dockerfile
# to force sqlx to look at the saved metadata
# instead of trying to query a live database
ENV SQLX_OFFLINE true

# Let's build our binary!
# We'll use the release profile to make it faaaast
RUN cargo build --release

# Runtime stage
FROM rust:1.59.0 AS runtime

WORKDIR /app

# Copy the compiled binary from the builder environment
# to our runtime environment
COPY --from=builder /app/target/release/newsletter newsletter

# We need the configuration file at runtime!
COPY configuration configuration

# Instruct the binary in our Docker image to
# use the production configuration
ENV APP_ENVIRONMENT production

# When `docker run` is executed, launch the binary!
ENTRYPOINT ["./newsletter"]