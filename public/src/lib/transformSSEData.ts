interface SSEResponse {
  voltage_array: number[];
  current_array: number[];
  is_progress: boolean;
}

export const getSSEProgStatus = (sseResponse: SSEResponse): boolean =>
  sseResponse.is_progress;

export const getSSEVoltageArray = (sseResponse: SSEResponse): number[][] =>
  sseResponse.voltage_array.reduce((acc: number[][], mod, index) => {
    if ((index + 1) % 4 !== 0) {
      if (acc.length === 0 || acc[acc.length - 1].length === 4) {
        acc.push([mod]);
      } else {
        acc[acc.length - 1].push(mod);
      }
    } else {
      acc[acc.length - 1].push(mod);
    }
    return acc;
  }, []);

export const getSSECurrentArray = (sseResponse: SSEResponse): number[][] =>
  sseResponse.current_array.reduce((acc: number[][], mod, index) => {
    if ((index + 1) % 4 !== 0) {
      if (acc.length === 0 || acc[acc.length - 1].length === 4) {
        acc.push([mod]);
      } else {
        acc[acc.length - 1].push(mod);
      }
    } else {
      acc[acc.length - 1].push(mod);
    }
    return acc;
  }, []);
