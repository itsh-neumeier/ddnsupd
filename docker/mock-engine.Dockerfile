FROM python:3.13-alpine
COPY tests/mock-engine.py /app/mock.py
USER 10001:10001
HEALTHCHECK --interval=2s --timeout=3s --retries=10 CMD ["python", "-c", "import urllib.request; urllib.request.urlopen('http://localhost:8090/healthz')"]
ENTRYPOINT ["python", "/app/mock.py"]
