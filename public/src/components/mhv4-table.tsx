"use client";

import React from "react";
import { useMHV4Data } from "@/contexts/MHV4Context";

import { Switch } from "@/components/ui/switch";
import { Input } from "@/components/ui/input";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

const processBooleanArray = (
  inputArray: boolean[],
  trueValue: string,
  falseValue: string
): string[] => {
  return inputArray.map(value => value ? trueValue : falseValue);
};

const processNumberArray = (
  inputArray: number[],
  transform: (value: number) => number,
  decimalPlaces: number
): string[] => {
  return inputArray.map(value =>
    value < -99999 ? "read error!" : transform(value).toFixed(decimalPlaces)
  );
};

const processPolArray = (inputArray: boolean[]): string[] => 
  processBooleanArray(inputArray, "+", "-");

const processOnOffArray = (inputArray: boolean[]): string[] => 
  processBooleanArray(inputArray, "ON", "OFF");

const processVoltageArray = (inputArray: number[]): string[] => 
  processNumberArray(inputArray, value => value * 0.1, 1);

const processCurrentArray = (inputArray: number[]): string[] => 
  processNumberArray(inputArray, value => value * 0.001, 3);

const MHV4Table: React.FC = () => {
  const { progressType, busArray, devArray, chArray, voltageArray, currentArray, isOnArray, isPositiveArray } = useMHV4Data();
  const onoffs = processOnOffArray(isOnArray);
  const pols = processPolArray(isPositiveArray);
  const voltages = processVoltageArray(voltageArray);
  const currents = processCurrentArray(currentArray);

  let border_style = "border-2 border-green-500";
  if (progressType) {
    border_style = "border-2 border-yellow-500";
  }

  return (
    <Table className={border_style}>
      <TableHeader className="bg-blue-100">
        <TableRow>
          <TableHead className="font-bold">bus</TableHead>
          <TableHead className="font-bold">dev</TableHead>
          <TableHead className="font-bold">ch</TableHead>
          <TableHead className="font-bold">status</TableHead>
          <TableHead className="font-bold">on/off</TableHead>
          <TableHead className="font-bold">pol</TableHead>
          <TableHead className="font-bold">input (V)</TableHead>
          <TableHead className="font-bold">voltage (V)</TableHead>
          <TableHead className="font-bold">current (uA)</TableHead>
          <TableHead className="font-bold">name</TableHead>
          <TableHead className="font-bold">discription</TableHead>
        </TableRow>
      </TableHeader>
      <TableBody>
        {busArray.map((bus, index) => (
          <TableRow key={bus}>
            <TableCell className="border">{bus}</TableCell>
            <TableCell className="border">{devArray[index]}</TableCell>
            <TableCell className="border">{chArray[index]}</TableCell>
            <TableCell className="border">{onoffs[index]}</TableCell>
            <TableCell className="border"><Switch/></TableCell>
            <TableCell className="border">{pols[index]}</TableCell>
            <TableCell className="border"><Input/></TableCell>
            <TableCell className="border">{voltages[index]}</TableCell>
            <TableCell className="border">{currents[index]}</TableCell>
            <TableCell className="border"><Input/></TableCell>
            <TableCell className="border"><Input/></TableCell>
          </TableRow>
        ))}
      </TableBody>

    </Table>
  )
}

export default MHV4Table;
