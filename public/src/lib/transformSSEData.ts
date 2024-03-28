type SSEType = [number[], number[], boolean];

export const getSSEProgStatus = (sseResponse: SSEType): boolean =>
  sseResponse[2];

export const getSSEVoltageArray = (sseResponse: SSEType): number[][] => {
  return sseResponse[0].reduce((acc: number[][], mod, index) => {
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
};

export const getSSECurrentArray = (sseResponse: SSEType): number[][] => {
  return sseResponse[1].reduce((acc: number[][], mod, index) => {
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
};
