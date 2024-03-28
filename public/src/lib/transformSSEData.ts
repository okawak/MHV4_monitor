type SSEType = [voltage: number[], current: number[], is_progress: boolean];

export const getSSEProgStatus = (sseResponse: SSEType): boolean =>
  sseResponse[2];

export const getSSEVoltageArray = (sseResponse: SSEType): number[] => 
  sseResponse[0];

export const getSSECurrentArray = (sseResponse: SSEType): number[] =>
  sseResponse[1];
