import torch
import zmq

import threading
import queue
import pickle
from typing import List

class ExternalDataset(torch.utils.data.IterableDataset):

    def __init__(self, address, batch_size=1024, fields:List[str] = None, maxsize=8):
        self.batch_size = batch_size
        self.fields = fields

        self.ctx = zmq.Context()
        self.socket = self.ctx.socket(zmq.REQ)
        self.socket.connect(address)

        # Get the Dataset info from the server
        self.socket.send_string("Info")
        data = self.socket.recv()
        self.info = pickle.loads(data)

        # Start the loading process
        load_thread = threading.Thread(target=self.__load_data__)
        self.data_queue = queue.Queue(maxsize = maxsize)
        load_thread.start()

        self.internal_iter = self.__internal_item__()

    def __load_data__(self):
        while True:
            self.socket.send_string("Data")
            data = self.socket.recv()
            if len(data) == 8:
                print("Done with Download")
                break
            result = pickle.loads(data)
            self.data_queue.put(result)

    def __iter__(self):
        return self

    def __len__(self):
        return self.info['length']

    def __next__(self):
        return next(self.internal_iter)

    def __internal_item__(self):
        while True:
            data = self.data_queue.get()

            for x in range(self.batch_size):
                result = dict()
                keys = data.keys()
                if self.fields is not None:
                    keys = self.fields
                for key in keys:
                    try:
                        result[key] = data[key][x]
                    except:
                        print("Errr")
                yield result

    def __getitem__(self, idx):
        return next(self.internal_iter)



def main():
    dataset = ExternalDataset("ipc:///tmp/multi-label")


if __name__ == '__main__':
    main()