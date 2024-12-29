type PeerStatus = "Online" | "Offline" | "Busy";
type PairStatus =
  | "NONE"
  | "REQUESTED"
  | "REQUEST_RECEIVED"
  | "PAIRED"
  | "REJECTED";

interface PeerInfo {
  addr: string;
  port: string;
  hostname: string;
  id: string;
  last_seen: number;
  status: PeerStatus;
  version: string;
  os_info: OsInformation;
}
