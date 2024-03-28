"use client";

import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  ReactNode,
} from "react";

import {
  getInitRCStatus,
  getInitProgStatus,
  getInitMHV4bus,
  getInitMHV4dev,
  getInitMHV4ch,
  getInitMHV4onoff,
  getInitMHV4pol,
} from "@/lib/transformInitData";

type RCType = boolean;
type ProgressType = boolean;
type BusType = number[][];
type DevType = number[][];
type ChType = number[][];
type VoltageType = number[][];
type CurrentType = number[][];
type IsOnType = boolean[][];
type IsPositiveType = boolean[][];

interface MHV4ContextType {
  rcType: RCType;
  progressType: ProgressType;
  busArray: BusType;
  devArray: DevType;
  chArray: ChType;
  voltageArray: VoltageType;
  currentArray: CurrentType;
  isOnArray: IsOnType;
  isPositiveArray: IsPositiveType;
}

const defaultState: MHV4ContextType = {
  rcType: false,
  progressType: false,
  busArray: [],
  devArray: [],
  chArray: [],
  voltageArray: [],
  currentArray: [],
  isOnArray: [],
  isPositiveArray: [],
};

const MHV4Context = createContext<MHV4ContextType>(defaultState);

interface MHV4ProviderProps {
  children: ReactNode;
}

export const MHV4DataProvider: React.FC<MHV4ProviderProps> = ({ children }) => {
  const [rcType, setRCType] = useState<RCType>(defaultState.rcType);
  const [progressType, setProgressType] = useState<ProgressType>(
    defaultState.progressType,
  );
  const [busArray, setBusArray] = useState<BusType>(defaultState.busArray);
  const [devArray, setDevArray] = useState<DevType>(defaultState.devArray);
  const [chArray, setChArray] = useState<ChType>(defaultState.chArray);

  const [voltageArray, setVolArray] = useState<VoltageType>(
    defaultState.voltageArray,
  );
  const [currentArray, setCurArray] = useState<CurrentType>(
    defaultState.currentArray,
  );

  const [isOnArray, setIsOnArray] = useState<IsOnType>(defaultState.isOnArray);
  const [isPositiveArray, setIsPositiveArray] = useState<IsPositiveType>(
    defaultState.isPositiveArray,
  );

  useEffect(() => {
    const fetchData = async () => {
      try {
        const response = await fetch(`${process.env.NEXT_PUBLIC_INIT_ROUTE}`);
        if (!response.ok) {
          throw new Error("failed to fetch the MHV4 data");
        }
        const initialData = await response.json();
        console.log(initialData);
        const data = JSON.parse(initialData);
        // set initial state
        console.log(getInitRCStatus(data));
        setRCType(getInitRCStatus(data));

        console.log(getInitProgStatus(data));
        setProgressType(getInitProgStatus(data));

        console.log(getInitMHV4bus(data));
        setBusArray(getInitMHV4bus(data));

        console.log(getInitMHV4dev(data));
        setDevArray(getInitMHV4dev(data));

        console.log(getInitMHV4ch(data));
        setChArray(getInitMHV4ch(data));

        console.log(getInitMHV4onoff(data));
        setIsOnArray(getInitMHV4onoff(data));

        console.log(getInitMHV4pol(data));
        setIsPositiveArray(getInitMHV4pol(data));
      } catch (error) {
        console.error("Failed to fetch initial data:", error);
      }
    };

    fetchData();

    const eventSource = new EventSource(`${process.env.NEXT_PUBLIC_SSE_ROUTE}`);
    eventSource.onopen = (event) => {
      console.log("SSE connection opened: ", event);
    };
    eventSource.onmessage = (event) => {
      console.log("SSE message received: ", event);
      const data = JSON.parse(event.data);
      // process
    };
    eventSource.onerror = (event) => {
      console.error("SSE connection error: ", event);
      // process
    };

    return () => {
      eventSource.close();
    };
  }, []);

  return (
    <MHV4Context.Provider
      value={{
        rcType,
        progressType,
        busArray,
        devArray,
        chArray,
        voltageArray,
        currentArray,
        isOnArray,
        isPositiveArray,
      }}
    >
      {children}
    </MHV4Context.Provider>
  );
};

export const useMHV4Data = (): MHV4ContextType => useContext(MHV4Context);
