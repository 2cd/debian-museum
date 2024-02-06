FROM curlimages/curl-base:8.6.0

WORKDIR /app
COPY tmp.get-base-disk /app/get-base-disk

CMD ["sh"]
