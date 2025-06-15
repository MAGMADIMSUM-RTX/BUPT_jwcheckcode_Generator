FROM ubuntu:latest

WORKDIR /app

COPY target/x86_64-unknown-linux-gnu/release/jw_code ./target/jw_code_amd64
COPY target/release/jw_code ./target/jw_code_arm64
COPY static/ ./static/
COPY lessons_data.db ./lessons_data.db
COPY entrypoint.sh ./entrypoint.sh

# RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/* \
RUN chmod +x ./target/jw_code_amd64 ./target/jw_code_arm64 ./entrypoint.sh

EXPOSE 2233

CMD ["./entrypoint.sh"]
