import logging
import sys

logging.basicConfig(format='%(asctime)s %(levelname)s %(funcName)s:%(lineno)d: %(message)s')

# Exit on critical log
class ShutdownHandler(logging.Handler):
    def emit(self, record):
        print(record, file=sys.stderr)
        logging.shutdown()
        sys.exit(1)

logger = logging.getLogger()
logger.addHandler(ShutdownHandler(level=logging.CRITICAL))
