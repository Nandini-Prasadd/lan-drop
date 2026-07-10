export type TransferDirection = 'incoming' | 'outgoing';

export type TransferState =
  | 'queued'
  | 'awaitingPairing'
  | 'transferring'
  | 'verifying'
  | 'completed'
  | 'cancelled'
  | 'failed';

export interface PeerMetadata {
  id: string;
  displayName: string;
}

export interface FileMetadata {
  name: string;
  sizeBytes: number;
  sha256: string | null;
}

export interface TransferMetadata {
  id: string;
  direction: TransferDirection;
  peer: PeerMetadata;
  file: FileMetadata;
  state: TransferState;
  createdAt: string;
}

export type MetadataValidationCode =
  | 'invalidPeerId'
  | 'invalidPeerName'
  | 'invalidFileName'
  | 'emptyFile'
  | 'fileTooLarge'
  | 'invalidChecksum';

export interface MetadataValidationError {
  code: MetadataValidationCode;
  message: string;
}

export interface MetadataValidationResult {
  valid: boolean;
  error: MetadataValidationError | null;
}

export type TransferFailureCode = 'networkUnavailable' | 'peerRejected' | 'diskFull' | 'integrityFailed' | 'validationFailed';

export interface TransferFailure {
  code: TransferFailureCode;
  message: string;
  retryable: boolean;
}
