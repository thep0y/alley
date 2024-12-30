interface TransferTask {
  id: string;
  peer_id: string;
  transfer_type: TransferType;
}

type TransferType = TransferFileType | TransferDirectoryType | TransferTextType;

interface TransferFileType {
  path: string;
  name: string;
  size: number;
  progress: number;
  status: TransferStatus;
}

interface TransferDirectoryType {
  path: string;
  total_files: number;
}

interface TransferTextType {
  content: string;
}
