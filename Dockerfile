FROM alpine:latest
MAINTAINER suti <lxy96@outlook.com>

EXPOSE 8210

RUN mkdir -p /usr/project/text-render-service /opt/chuangkit.font.cache /tmp/text-log/
WORKDIR /usr/project/text-render-service

COPY ./release /usr/project/text-render-service/release

CMD ./release/server > /tmp/text-log/info.log 2> /tmp/text-log/error.log
