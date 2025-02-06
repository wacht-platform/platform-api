FROM rust:1.84.0-alpine as build

WORKDIR /app

# Copy our application source
COPY . /app

# Build the application
RUN cargo build --release

FROM alpine:latest

WORKDIR /app

# Copy the released application from the previous image
COPY --from=build /app/target/release/dashboard-api /app/

# Expose the port the application will run on
EXPOSE 3000

# Run the application
CMD ["./dashboard-api"]
