"""Stream public snapshots with CPython's JSON encoder in scalar batches."""
import json


def dump_public_json(value, stream):
    # json.dump uses the Python encoder one token/write call at a time. Keep
    # its exact compact schema and number rules, but let the C encoder handle
    # each bounded scalar batch rather than materializing a whole snapshot.
    encoder = json.JSONEncoder(allow_nan=False, separators=(',', ':'))
    active = set()

    def emit(item):
        container = isinstance(item, (dict, list, tuple))
        if not container:
            stream.write(encoder.encode(item))
            return
        identity = id(item)
        if identity in active:
            raise ValueError('Circular reference detected')
        active.add(identity)
        try:
            if isinstance(item, dict):
                if not all(isinstance(key, str) for key in item):
                    stream.write(encoder.encode(item))
                    return
                stream.write('{')
                for index, (key, child) in enumerate(item.items()):
                    if index:
                        stream.write(',')
                    stream.write(encoder.encode(key))
                    stream.write(':')
                    emit(child)
                stream.write('}')
                return
            stream.write('[')
            first = True
            batch = []

            def flush():
                nonlocal first
                if batch:
                    if not first:
                        stream.write(',')
                    stream.write(encoder.encode(batch)[1:-1])
                    first = False
                    batch.clear()

            for child in item:
                if isinstance(child, (dict, list, tuple)):
                    flush()
                    if not first:
                        stream.write(',')
                    emit(child)
                    first = False
                else:
                    batch.append(child)
                    if len(batch) == 8192:
                        flush()
            flush()
            stream.write(']')
        finally:
            active.remove(identity)

    emit(value)
