FROM registry.fedoraproject.org/fedora:latest

RUN dnf install -y nispor
CMD ["npc", "full"]

