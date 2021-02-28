/**
 * File name from path (retains extension). Supports Windows, Linux, Unix
 * Input: "/foo/bar/test.txt"
 * Output: "test.txt"
 */
export const fileNameFromPath = (path: string): string => {
  return path.replace(/^.*[\\/]/, "");
};
