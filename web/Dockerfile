FROM node:16

ARG uid
ARG gid

# Add user and group
RUN \
  userdel -r node && \
  groupadd -g ${gid} user && \
  useradd -m -u ${uid} -g ${gid} user
USER user:user
