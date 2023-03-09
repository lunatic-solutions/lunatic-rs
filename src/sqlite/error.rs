use serde::{Deserialize, Serialize};

use super::client::SqliteClient;
use crate::host::call_host_alloc;

/// An Sqlite error with a code and optional message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SqliteError {
    /// Error code.
    pub code: SqliteCode,
    /// Error message.
    pub message: Option<String>,
}

// Generated with the following JS code on https://www.sqlite.org/rescode.html in developer console.
// ```js
// // Paste JS from https://unpkg.com/turndown@7.1.1/dist/turndown.js
//
// // Initialize turndown service
// var turndownService = new TurndownService()
// TurndownService.prototype.escape = function(s) { return s };
//
// const snakeToCamel = str =>
//   str.toLowerCase().replace(/([-_][a-z])/g, group =>
//   group
//     .toUpperCase()
//     .replace('-', '')
//     .replace('_', '')
// );
//
// const errorsContainer = document.querySelector("body > div.fancy");
// const errors = [];
// for (let i = 0; i < errorsContainer.children.length; i++) {
//   const el = errorsContainer.children[i];
//   switch (el.tagName) {
//       case 'H3':
//           {
//               let [code, name] = el.textContent.split(')');
//               code = parseInt(code.substring(1));
//               name = name.substring(1);
//               errors.push({ code, name, docs: '' });
//               break;
//           }
//       default:
//           {
//               const md = turndownService.turndown(el.outerHTML);
//               if (errors[errors.length - 1].docs) {
//                   errors[errors.length - 1].docs += '\n\n';
//               }
//               errors[errors.length - 1].docs += `${md}`;
//           }
//   }
// }
//
// let s = '';
// for (let i = 0; i < errors.length; i++) {
//   const error = errors[i];
//   const docs = '/// ' + error.docs.replace(/\n/g, '\n/// ');
//   let name = snakeToCamel(error.name);
//   if (name.startsWith('sqlite')) {
//       name = name.substring(6);
//   }
//   s += `${docs}\n`;
//   s += `${name} = ${error.code},\n`;
// }
// console.log(s);
// ```
/// Sqlite codes as seen at [https://www.sqlite.org/rescode.html].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum SqliteCode {
    /// The SQLITE_OK result code means that the operation was successful and
    /// that there were no errors. Most other result codes indicate an error.
    Ok = 0,
    /// The SQLITE_ERROR result code is a generic error code that is used when
    /// no other more specific error code is available.
    Error = 1,
    /// The SQLITE_INTERNAL result code indicates an internal malfunction. In a
    /// working version of SQLite, an application should never see this result
    /// code. If application does encounter this result code, it shows that
    /// there is a bug in the database engine.
    ///
    /// SQLite does not currently generate this result code. However, [application-defined SQL functions](https://www.sqlite.org/appfunc.html) or [virtual tables](https://www.sqlite.org/vtab.html), or [VFSes](https://www.sqlite.org/vfs.html), or other extensions might cause this result code to be returned.
    Internal = 2,
    /// The SQLITE_PERM result code indicates that the requested access mode for
    /// a newly created database could not be provided.
    Perm = 3,
    /// The SQLITE_ABORT result code indicates that an operation was aborted prior to completion, usually be application request. See also: [SQLITE_INTERRUPT](https://www.sqlite.org/rescode.html#interrupt).
    ///
    /// If the callback function to [sqlite3_exec()](https://www.sqlite.org/c3ref/exec.html) returns non-zero, then sqlite3_exec() will return SQLITE_ABORT.
    ///
    /// If a [ROLLBACK](https://www.sqlite.org/lang_transaction.html) operation occurs on the same [database connection](https://www.sqlite.org/c3ref/sqlite3.html) as a pending read or write, then the pending read or write may fail with an SQLITE_ABORT or [SQLITE_ABORT_ROLLBACK](https://www.sqlite.org/rescode.html#abort_rollback) error.
    ///
    /// In addition to being a result code, the SQLITE_ABORT value is also used as a [conflict resolution mode](https://www.sqlite.org/c3ref/c_fail.html) returned from the [sqlite3_vtab_on_conflict()](https://www.sqlite.org/c3ref/vtab_on_conflict.html) interface.
    Abort = 4,
    /// The SQLITE_BUSY result code indicates that the database file could not be written (or in some cases read) because of concurrent activity by some other [database connection](https://www.sqlite.org/c3ref/sqlite3.html), usually a database connection in a separate process.
    ///
    /// For example, if process A is in the middle of a large write transaction and at the same time process B attempts to start a new write transaction, process B will get back an SQLITE_BUSY result because SQLite only supports one writer at a time. Process B will need to wait for process A to finish its transaction before starting a new transaction. The [sqlite3_busy_timeout()](https://www.sqlite.org/c3ref/busy_timeout.html) and [sqlite3_busy_handler()](https://www.sqlite.org/c3ref/busy_handler.html) interfaces and the [busy_timeout pragma](https://www.sqlite.org/pragma.html#pragma_busy_timeout) are available to process B to help it deal with SQLITE_BUSY errors.
    ///
    /// An SQLITE_BUSY error can occur at any point in a transaction: when the transaction is first started, during any write or update operations, or when the transaction commits. To avoid encountering SQLITE_BUSY errors in the middle of a transaction, the application can use [BEGIN IMMEDIATE](https://www.sqlite.org/lang_transaction.html#immediate) instead of just [BEGIN](https://www.sqlite.org/lang_transaction.html) to start a transaction. The [BEGIN IMMEDIATE](https://www.sqlite.org/lang_transaction.html#immediate) command might itself return SQLITE_BUSY, but if it succeeds, then SQLite guarantees that no subsequent operations on the same database through the next [COMMIT](https://www.sqlite.org/lang_transaction.html) will return SQLITE_BUSY.
    ///
    /// See also: [SQLITE_BUSY_RECOVERY](https://www.sqlite.org/rescode.html#busy_recovery) and [SQLITE_BUSY_SNAPSHOT](https://www.sqlite.org/rescode.html#busy_snapshot).
    ///
    /// The SQLITE_BUSY result code differs from [SQLITE_LOCKED](https://www.sqlite.org/rescode.html#locked) in that SQLITE_BUSY indicates a conflict with a separate [database connection](https://www.sqlite.org/c3ref/sqlite3.html), probably in a separate process, whereas [SQLITE_LOCKED](https://www.sqlite.org/rescode.html#locked) indicates a conflict within the same [database connection](https://www.sqlite.org/c3ref/sqlite3.html) (or sometimes a database connection with a [shared cache](https://www.sqlite.org/sharedcache.html)).
    Busy = 5,
    /// The SQLITE_LOCKED result code indicates that a write operation could not continue because of a conflict within the same [database connection](https://www.sqlite.org/c3ref/sqlite3.html) or a conflict with a different database connection that uses a [shared cache](https://www.sqlite.org/sharedcache.html).
    ///
    /// For example, a [DROP TABLE](https://www.sqlite.org/lang_droptable.html) statement cannot be run while another thread is reading from that table on the same [database connection](https://www.sqlite.org/c3ref/sqlite3.html) because dropping the table would delete the table out from under the concurrent reader.
    ///
    /// The SQLITE_LOCKED result code differs from [SQLITE_BUSY](https://www.sqlite.org/rescode.html#busy) in that SQLITE_LOCKED indicates a conflict on the same [database connection](https://www.sqlite.org/c3ref/sqlite3.html) (or on a connection with a [shared cache](https://www.sqlite.org/sharedcache.html)) whereas [SQLITE_BUSY](https://www.sqlite.org/rescode.html#busy) indicates a conflict with a different database connection, probably in a different process.
    Locked = 6,
    /// The SQLITE_NOMEM result code indicates that SQLite was unable to allocate all the memory it needed to complete the operation. In other words, an internal call to [sqlite3_malloc()](https://www.sqlite.org/c3ref/free.html) or [sqlite3_realloc()](https://www.sqlite.org/c3ref/free.html) has failed in a case where the memory being allocated was required in order to continue the operation.
    Nomem = 7,
    /// The SQLITE_READONLY result code is returned when an attempt is made to
    /// alter some data for which the current database connection does not have
    /// write permission.
    Readonly = 8,
    /// The SQLITE_INTERRUPT result code indicates that an operation was interrupted by the [sqlite3_interrupt()](https://www.sqlite.org/c3ref/interrupt.html) interface. See also: [SQLITE_ABORT](https://www.sqlite.org/rescode.html#abort)
    Interrupt = 9,
    /// The SQLITE_IOERR result code says that the operation could not finish
    /// because the operating system reported an I/O error.
    ///
    /// A full disk drive will normally give an [SQLITE_FULL](https://www.sqlite.org/rescode.html#full) error rather than an SQLITE_IOERR error.
    ///
    /// There are many different extended result codes for I/O errors that
    /// identify the specific I/O operation that failed.
    Ioerr = 10,
    /// The SQLITE_CORRUPT result code indicates that the database file has been corrupted. See the [How To Corrupt Your Database Files](https://www.sqlite.org/lockingv3.html#how_to_corrupt) for further discussion on how corruption can occur.
    Corrupt = 11,
    /// The SQLITE_NOTFOUND result code is exposed in three ways:
    ///
    /// 1.  SQLITE_NOTFOUND can be returned by the [sqlite3_file_control()](https://www.sqlite.org/c3ref/file_control.html) interface to indicate that the [file control opcode](https://www.sqlite.org/c3ref/c_fcntl_begin_atomic_write.html) passed as the third argument was not recognized by the underlying [VFS](https://www.sqlite.org/vfs.html).
    ///     
    /// 2.  SQLITE_NOTFOUND can also be returned by the xSetSystemCall() method of an [sqlite3_vfs](https://www.sqlite.org/c3ref/vfs.html) object.
    ///     
    /// 3.  SQLITE_NOTFOUND an be returned by [sqlite3_vtab_rhs_value()](https://www.sqlite.org/c3ref/vtab_rhs_value.html) to indicate that the right-hand operand of a constraint is not available to the [xBestIndex method](https://www.sqlite.org/vtab.html#xbestindex) that made the call.
    ///
    /// The SQLITE_NOTFOUND result code is also used internally by the SQLite
    /// implementation, but those internal uses are not exposed to the
    /// application.
    Notfound = 12,
    /// The SQLITE_FULL result code indicates that a write could not complete because the disk is full. Note that this error can occur when trying to write information into the main database file, or it can also occur when writing into [temporary disk files](https://www.sqlite.org/tempfiles.html).
    ///
    /// Sometimes applications encounter this error even though there is an abundance of primary disk space because the error occurs when writing into [temporary disk files](https://www.sqlite.org/tempfiles.html) on a system where temporary files are stored on a separate partition with much less space that the primary disk.
    Full = 13,
    /// The SQLITE_CANTOPEN result code indicates that SQLite was unable to open a file. The file in question might be a primary database file or one of several [temporary disk files](https://www.sqlite.org/tempfiles.html).
    Cantopen = 14,
    /// The SQLITE_PROTOCOL result code indicates a problem with the file locking protocol used by SQLite. The SQLITE_PROTOCOL error is currently only returned when using [WAL mode](https://www.sqlite.org/wal.html) and attempting to start a new transaction. There is a race condition that can occur when two separate [database connections](https://www.sqlite.org/c3ref/sqlite3.html) both try to start a transaction at the same time in [WAL mode](https://www.sqlite.org/wal.html). The loser of the race backs off and tries again, after a brief delay. If the same connection loses the locking race dozens of times over a span of multiple seconds, it will eventually give up and return SQLITE_PROTOCOL. The SQLITE_PROTOCOL error should appear in practice very, very rarely, and only when there are many separate processes all competing intensely to write to the same database.
    Protocol = 15,
    /// The SQLITE_EMPTY result code is not currently used.
    Empty = 16,
    /// The SQLITE_SCHEMA result code indicates that the database schema has changed. This result code can be returned from [sqlite3_step()](https://www.sqlite.org/c3ref/step.html) for a [prepared statement](https://www.sqlite.org/c3ref/stmt.html) that was generated using [sqlite3_prepare()](https://www.sqlite.org/c3ref/prepare.html) or [sqlite3_prepare16()](https://www.sqlite.org/c3ref/prepare.html). If the database schema was changed by some other process in between the time that the statement was prepared and the time the statement was run, this error can result.
    ///
    /// If a [prepared statement](https://www.sqlite.org/c3ref/stmt.html) is generated from [sqlite3_prepare_v2()](https://www.sqlite.org/c3ref/prepare.html) then the statement is automatically re-prepared if the schema changes, up to [SQLITE_MAX_SCHEMA_RETRY](https://www.sqlite.org/compile.html#max_schema_retry) times (default: 50). The [sqlite3_step()](https://www.sqlite.org/c3ref/step.html) interface will only return SQLITE_SCHEMA back to the application if the failure persists after these many retries.
    Schema = 17,
    /// The SQLITE_TOOBIG error code indicates that a string or BLOB was too large. The default maximum length of a string or BLOB in SQLite is 1,000,000,000 bytes. This maximum length can be changed at compile-time using the [SQLITE_MAX_LENGTH](https://www.sqlite.org/limits.html#max_length) compile-time option, or at run-time using the [sqlite3_limit](https://www.sqlite.org/c3ref/limit.html)(db,[SQLITE_LIMIT_LENGTH](https://www.sqlite.org/c3ref/c_limit_attached.html#sqlitelimitlength),...) interface. The SQLITE_TOOBIG error results when SQLite encounters a string or BLOB that exceeds the compile-time or run-time limit.
    ///
    /// The SQLITE_TOOBIG error code can also result when an oversized SQL statement is passed into one of the [sqlite3_prepare_v2()](https://www.sqlite.org/c3ref/prepare.html) interfaces. The maximum length of an SQL statement defaults to a much smaller value of 1,000,000,000 bytes. The maximum SQL statement length can be set at compile-time using [SQLITE_MAX_SQL_LENGTH](https://www.sqlite.org/limits.html#max_sql_length) or at run-time using [sqlite3_limit](https://www.sqlite.org/c3ref/limit.html)(db,[SQLITE_LIMIT_SQL_LENGTH](https://www.sqlite.org/c3ref/c_limit_attached.html#sqlitelimitsqllength),...).
    Toobig = 18,
    /// The SQLITE_CONSTRAINT error code means that an SQL constraint violation occurred while trying to process an SQL statement. Additional information about the failed constraint can be found by consulting the accompanying error message (returned via [sqlite3_errmsg()](https://www.sqlite.org/c3ref/errcode.html) or [sqlite3_errmsg16()](https://www.sqlite.org/c3ref/errcode.html)) or by looking at the [extended error code](https://www.sqlite.org/rescode.html#extrc).
    ///
    /// The SQLITE_CONSTRAINT code can also be used as the return value from the [xBestIndex()](https://www.sqlite.org/vtab.html#xbestindex) method of a [virtual table](https://www.sqlite.org/vtab.html) implementation. When xBestIndex() returns SQLITE_CONSTRAINT, that indicates that the particular combination of inputs submitted to xBestIndex() cannot result in a usable query plan and should not be given further consideration.
    Constraint = 19,
    /// The SQLITE_MISMATCH error code indicates a datatype mismatch.
    ///
    /// SQLite is normally very forgiving about mismatches between the type of a
    /// value and the declared type of the container in which that value is to
    /// be stored. For example, SQLite allows the application to store a large
    /// BLOB in a column with a declared type of BOOLEAN. But in a few cases,
    /// SQLite is strict about types. The SQLITE_MISMATCH error is returned in
    /// those few cases when the types do not match.
    ///
    /// The [rowid](https://www.sqlite.org/lang_createtable.html#rowid) of a table must be an integer. Attempt to set the [rowid](https://www.sqlite.org/lang_createtable.html#rowid) to anything other than an integer (or a NULL which will be automatically converted into the next available integer rowid) results in an SQLITE_MISMATCH error.
    Mismatch = 20,
    /// The SQLITE_MISUSE return code might be returned if the application uses any SQLite interface in a way that is undefined or unsupported. For example, using a [prepared statement](https://www.sqlite.org/c3ref/stmt.html) after that prepared statement has been [finalized](https://www.sqlite.org/c3ref/finalize.html) might result in an SQLITE_MISUSE error.
    ///
    /// SQLite tries to detect misuse and report the misuse using this result
    /// code. However, there is no guarantee that the detection of misuse will
    /// be successful. Misuse detection is probabilistic. Applications should
    /// never depend on an SQLITE_MISUSE return value.
    ///
    /// If SQLite ever returns SQLITE_MISUSE from any interface, that means that
    /// the application is incorrectly coded and needs to be fixed. Do not ship
    /// an application that sometimes returns SQLITE_MISUSE from a standard
    /// SQLite interface because that application contains potentially serious
    /// bugs.
    Misuse = 21,
    /// The SQLITE_NOLFS error can be returned on systems that do not support
    /// large files when the database grows to be larger than what the
    /// filesystem can handle. "NOLFS" stands for "NO Large File Support".
    Nolfs = 22,
    /// The SQLITE_AUTH error is returned when the [authorizer callback](https://www.sqlite.org/c3ref/set_authorizer.html) indicates that an SQL statement being prepared is not authorized.
    Auth = 23,
    /// The SQLITE_FORMAT error code is not currently used by SQLite.
    Format = 24,
    /// The SQLITE_RANGE error indices that the parameter number argument to one of the [sqlite3_bind](https://www.sqlite.org/c3ref/bind_blob.html) routines or the column number in one of the [sqlite3_column](https://www.sqlite.org/c3ref/column_blob.html) routines is out of range.
    Range = 25,
    /// When attempting to open a file, the SQLITE_NOTADB error indicates that
    /// the file being opened does not appear to be an SQLite database file.
    Notadb = 26,
    /// The SQLITE_NOTICE result code is not returned by any C/C++ interface. However, SQLITE_NOTICE (or rather one of its [extended error codes](https://www.sqlite.org/rescode.html#extrc)) is sometimes used as the first argument in an [sqlite3_log()](https://www.sqlite.org/c3ref/log.html) callback to indicate that an unusual operation is taking place.
    Notice = 27,
    /// The SQLITE_WARNING result code is not returned by any C/C++ interface. However, SQLITE_WARNING (or rather one of its [extended error codes](https://www.sqlite.org/rescode.html#extrc)) is sometimes used as the first argument in an [sqlite3_log()](https://www.sqlite.org/c3ref/log.html) callback to indicate that an unusual and possibly ill-advised operation is taking place.
    Warning = 28,
    /// The SQLITE_ROW result code returned by [sqlite3_step()](https://www.sqlite.org/c3ref/step.html) indicates that another row of output is available.
    Row = 100,
    /// The SQLITE_DONE result code indicates that an operation has completed. The SQLITE_DONE result code is most commonly seen as a return value from [sqlite3_step()](https://www.sqlite.org/c3ref/step.html) indicating that the SQL statement has run to completion. But SQLITE_DONE can also be returned by other multi-step interfaces such as [sqlite3_backup_step()](https://www.sqlite.org/c3ref/backup_finish.html#sqlite3backupstep).
    Done = 101,
    /// The [sqlite3_load_extension()](https://www.sqlite.org/c3ref/load_extension.html) interface loads an [extension](https://www.sqlite.org/loadext.html) into a single database connection. The default behavior is for that extension to be automatically unloaded when the database connection closes. However, if the extension entry point returns SQLITE_OK_LOAD_PERMANENTLY instead of SQLITE_OK, then the extension remains loaded into the process address space after the database connection closes. In other words, the xDlClose methods of the [sqlite3_vfs](https://www.sqlite.org/c3ref/vfs.html) object is not called for the extension when the database connection closes.
    ///
    /// The SQLITE_OK_LOAD_PERMANENTLY return code is useful to [loadable extensions](https://www.sqlite.org/loadext.html) that register new [VFSes](https://www.sqlite.org/vfs.html), for example.
    OkLoadPermanently = 256,
    /// The SQLITE_ERROR_MISSING_COLLSEQ result code means that an SQL statement
    /// could not be prepared because a collating sequence named in that SQL
    /// statement could not be located.
    ///
    /// Sometimes when this error code is encountered, the [sqlite3_prepare_v2()](https://www.sqlite.org/c3ref/prepare.html) routine will convert the error into [SQLITE_ERROR_RETRY](https://www.sqlite.org/rescode.html#error_retry) and try again to prepare the SQL statement using a different query plan that does not require the use of the unknown collating sequence.
    ErrorMissingCollseq = 257,
    /// The SQLITE_BUSY_RECOVERY error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_BUSY](https://www.sqlite.org/rescode.html#busy) that indicates that an operation could not continue because another process is busy recovering a [WAL mode](https://www.sqlite.org/wal.html) database file following a crash. The SQLITE_BUSY_RECOVERY error code only occurs on [WAL mode](https://www.sqlite.org/wal.html) databases.
    BusyRecovery = 261,
    /// The SQLITE_LOCKED_SHAREDCACHE result code indicates that access to an SQLite data record is blocked by another database connection that is using the same record in [shared cache mode](https://www.sqlite.org/sharedcache.html). When two or more database connections share the same cache and one of the connections is in the middle of modifying a record in that cache, then other connections are blocked from accessing that data while the modifications are on-going in order to prevent the readers from seeing a corrupt or partially completed change.
    LockedSharedcache = 262,
    /// The SQLITE_READONLY_RECOVERY error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_READONLY](https://www.sqlite.org/rescode.html#readonly). The SQLITE_READONLY_RECOVERY error code indicates that a [WAL mode](https://www.sqlite.org/wal.html) database cannot be opened because the database file needs to be recovered and recovery requires write access but only read access is available.
    ReadonlyRecovery = 264,
    /// The SQLITE_IOERR_READ error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the [VFS](https://www.sqlite.org/vfs.html) layer while trying to read from a file on disk. This error might result from a hardware malfunction or because a filesystem came unmounted while the file was open.
    IoerrRead = 266,
    /// The SQLITE_CORRUPT_VTAB error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CORRUPT](https://www.sqlite.org/rescode.html#corrupt) used by [virtual tables](https://www.sqlite.org/vtab.html). A [virtual table](https://www.sqlite.org/vtab.html) might return SQLITE_CORRUPT_VTAB to indicate that content in the virtual table is corrupt.
    CorruptVtab = 267,
    /// The SQLITE_CANTOPEN_NOTEMPDIR error code is no longer used.
    CantopenNotempdir = 270,
    /// The SQLITE_CONSTRAINT_CHECK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [CHECK constraint](https://www.sqlite.org/lang_createtable.html#ckconst) failed.
    ConstraintCheck = 275,
    /// The SQLITE_AUTH_USER error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_AUTH](https://www.sqlite.org/rescode.html#auth) indicating that an operation was attempted on a database for which the logged in user lacks sufficient authorization.
    AuthUser = 279,
    /// The SQLITE_NOTICE_RECOVER_WAL result code is passed to the callback of [sqlite3_log()](https://www.sqlite.org/c3ref/log.html) when a [WAL mode](https://www.sqlite.org/wal.html) database file is recovered.
    NoticeRecoverWal = 283,
    /// The SQLITE_WARNING_AUTOINDEX result code is passed to the callback of [sqlite3_log()](https://www.sqlite.org/c3ref/log.html) whenever [automatic indexing](https://www.sqlite.org/optoverview.html#autoindex) is used. This can serve as a warning to application designers that the database might benefit from additional indexes.
    WarningAutoindex = 284,
    /// The SQLITE_ERROR_RETRY is used internally to provoke [sqlite3_prepare_v2()](https://www.sqlite.org/c3ref/prepare.html) (or one of its sibling routines for creating prepared statements) to try again to prepare a statement that failed with an error on the previous attempt.
    ErrorRetry = 513,
    /// The SQLITE_ABORT_ROLLBACK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_ABORT](https://www.sqlite.org/rescode.html#abort) indicating that an SQL statement aborted because the transaction that was active when the SQL statement first started was rolled back. Pending write operations always fail with this error when a rollback occurs. A [ROLLBACK](https://www.sqlite.org/lang_transaction.html) will cause a pending read operation to fail only if the schema was changed within the transaction being rolled back.
    AbortRollback = 516,
    /// The SQLITE_BUSY_SNAPSHOT error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_BUSY](https://www.sqlite.org/rescode.html#busy) that occurs on [WAL mode](https://www.sqlite.org/wal.html) databases when a database connection tries to promote a read transaction into a write transaction but finds that another [database connection](https://www.sqlite.org/c3ref/sqlite3.html) has already written to the database and thus invalidated prior reads.
    ///
    /// The following scenario illustrates how an SQLITE_BUSY_SNAPSHOT error
    /// might arise:
    ///
    /// 1.  Process A starts a read transaction on the database and does one or
    /// more SELECT statement. Process A keeps the transaction open.
    /// 2.  Process B updates the database, changing values previous read by
    /// process A. 3.  Process A now tries to write to the database. But
    /// process A's view of the database content is now obsolete because process
    /// B has modified the database file after process A read from it. Hence
    /// process A gets an SQLITE_BUSY_SNAPSHOT error.
    BusySnapshot = 517,
    /// The SQLITE_LOCKED_VTAB result code is not used by the SQLite core, but
    /// it is available for use by extensions. Virtual table implementations can
    /// return this result code to indicate that they cannot complete the
    /// current operation because of locks held by other threads or processes.
    ///
    /// The [R-Tree extension](https://www.sqlite.org/rtree.html) returns this result code when an attempt is made to update the R-Tree while another prepared statement is actively reading the R-Tree. The update cannot proceed because any change to an R-Tree might involve reshuffling and rebalancing of nodes, which would disrupt read cursors, causing some rows to be repeated and other rows to be omitted.
    LockedVtab = 518,
    /// The SQLITE_READONLY_CANTLOCK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_READONLY](https://www.sqlite.org/rescode.html#readonly). The SQLITE_READONLY_CANTLOCK error code indicates that SQLite is unable to obtain a read lock on a [WAL mode](https://www.sqlite.org/wal.html) database because the shared-memory file associated with that database is read-only.
    ReadonlyCantlock = 520,
    /// The SQLITE_IOERR_SHORT_READ error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating that a read attempt in the [VFS](https://www.sqlite.org/vfs.html) layer was unable to obtain as many bytes as was requested. This might be due to a truncated file.
    IoerrShortRead = 522,
    /// The SQLITE_CORRUPT_SEQUENCE result code means that the schema of the sqlite_sequence table is corrupt. The sqlite_sequence table is used to help implement the [AUTOINCREMENT](https://www.sqlite.org/autoinc.html) feature. The sqlite_sequence table should have the following format:
    ///
    /// > CREATE TABLE sqlite_sequence(name,seq);
    /// >
    ///
    /// If SQLite discovers that the sqlite_sequence table has any other format,
    /// it returns the SQLITE_CORRUPT_SEQUENCE error.
    CorruptSequence = 523,
    /// The SQLITE_CANTOPEN_ISDIR error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CANTOPEN](https://www.sqlite.org/rescode.html#cantopen) indicating that a file open operation failed because the file is really a directory.
    CantopenIsdir = 526,
    /// The SQLITE_CONSTRAINT_COMMITHOOK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [commit hook callback](https://www.sqlite.org/c3ref/commit_hook.html) returned non-zero that thus caused the SQL statement to be rolled back.
    ConstraintCommithook = 531,
    /// The SQLITE_NOTICE_RECOVER_ROLLBACK result code is passed to the callback of [sqlite3_log()](https://www.sqlite.org/c3ref/log.html) when a [hot journal](https://www.sqlite.org/fileformat2.html#hotjrnl) is rolled back.
    NoticeRecoverRollback = 539,
    /// The SQLITE_ERROR_SNAPSHOT result code might be returned when attempting to start a read transaction on an historical version of the database by using the [sqlite3_snapshot_open()](https://www.sqlite.org/c3ref/snapshot_open.html) interface. If the historical snapshot is no longer available, then the read transaction will fail with the SQLITE_ERROR_SNAPSHOT. This error code is only possible if SQLite is compiled with [-DSQLITE_ENABLE_SNAPSHOT](https://www.sqlite.org/compile.html#enable_snapshot).
    ErrorSnapshot = 769,
    /// The SQLITE_BUSY_TIMEOUT error code indicates that a blocking Posix
    /// advisory file lock request in the VFS layer failed due to a timeout.
    /// Blocking Posix advisory locks are only available as a proprietary SQLite
    /// extension and even then are only supported if SQLite is compiled with
    /// the SQLITE_EANBLE_SETLK_TIMEOUT compile-time option.
    BusyTimeout = 773,
    /// The SQLITE_READONLY_ROLLBACK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_READONLY](https://www.sqlite.org/rescode.html#readonly). The SQLITE_READONLY_ROLLBACK error code indicates that a database cannot be opened because it has a [hot journal](https://www.sqlite.org/fileformat2.html#hotjrnl) that needs to be rolled back but cannot because the database is readonly.
    ReadonlyRollback = 776,
    /// The SQLITE_IOERR_WRITE error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the [VFS](https://www.sqlite.org/vfs.html) layer while trying to write into a file on disk. This error might result from a hardware malfunction or because a filesystem came unmounted while the file was open. This error should not occur if the filesystem is full as there is a separate error code (SQLITE_FULL) for that purpose.
    IoerrWrite = 778,
    /// The SQLITE_CORRUPT_INDEX result code means that SQLite detected an entry is or was missing from an index. This is a special case of the [SQLITE_CORRUPT](https://www.sqlite.org/rescode.html#corrupt) error code that suggests that the problem might be resolved by running the [REINDEX](https://www.sqlite.org/lang_reindex.html) command, assuming no other problems exist elsewhere in the database file.
    CorruptIndex = 779,
    /// The SQLITE_CANTOPEN_FULLPATH error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CANTOPEN](https://www.sqlite.org/rescode.html#cantopen) indicating that a file open operation failed because the operating system was unable to convert the filename into a full pathname.
    CantopenFullpath = 782,
    /// The SQLITE_CONSTRAINT_FOREIGNKEY error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [foreign key constraint](https://www.sqlite.org/foreignkeys.html) failed.
    ConstraintForeignkey = 787,
    /// The SQLITE_READONLY_DBMOVED error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_READONLY](https://www.sqlite.org/rescode.html#readonly). The SQLITE_READONLY_DBMOVED error code indicates that a database cannot be modified because the database file has been moved since it was opened, and so any attempt to modify the database might result in database corruption if the processes crashes because the [rollback journal](https://www.sqlite.org/lockingv3.html#rollback) would not be correctly named.
    ReadonlyDbmoved = 1032,
    /// The SQLITE_IOERR_FSYNC error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the [VFS](https://www.sqlite.org/vfs.html) layer while trying to flush previously written content out of OS and/or disk-control buffers and into persistent storage. In other words, this code indicates a problem with the fsync() system call in unix or the FlushFileBuffers() system call in windows.
    IoerrFsync = 1034,
    /// The SQLITE_CANTOPEN_CONVPATH error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CANTOPEN](https://www.sqlite.org/rescode.html#cantopen) used only by Cygwin [VFS](https://www.sqlite.org/vfs.html) and indicating that the cygwin_conv_path() system call failed while trying to open a file. See also: [SQLITE_IOERR_CONVPATH](https://www.sqlite.org/rescode.html#ioerr_convpath)
    CantopenConvpath = 1038,
    /// The SQLITE_CONSTRAINT_FUNCTION error code is not currently used by the
    /// SQLite core. However, this error code is available for use by extension
    /// functions.
    ConstraintFunction = 1043,
    /// The SQLITE_READONLY_CANTINIT result code originates in the xShmMap method of a [VFS](https://www.sqlite.org/vfs.html) to indicate that the shared memory region used by [WAL mode](https://www.sqlite.org/wal.html) exists buts its content is unreliable and unusable by the current process since the current process does not have write permission on the shared memory region. (The shared memory region for WAL mode is normally a file with a "-wal" suffix that is mmapped into the process space. If the current process does not have write permission on that file, then it cannot write into shared memory.)
    ///
    /// Higher level logic within SQLite will normally intercept the error code
    /// and create a temporary in-memory shared memory region so that the
    /// current process can at least read the content of the database. This
    /// result code should not reach the application interface layer.
    ReadonlyCantinit = 1288,
    /// The SQLITE_IOERR_DIR_FSYNC error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the [VFS](https://www.sqlite.org/vfs.html) layer while trying to invoke fsync() on a directory. The unix [VFS](https://www.sqlite.org/vfs.html) attempts to fsync() directories after creating or deleting certain files to ensure that those files will still appear in the filesystem following a power loss or system crash. This error code indicates a problem attempting to perform that fsync().
    IoerrDirFsync = 1290,
    /// The SQLITE_CANTOPEN_DIRTYWAL result code is not used at this time.
    CantopenDirtywal = 1294,
    /// The SQLITE_CONSTRAINT_NOTNULL error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [NOT NULL constraint](https://www.sqlite.org/lang_createtable.html#notnullconst) failed.
    ConstraintNotnull = 1299,
    /// The SQLITE_READONLY_DIRECTORY result code indicates that the database is
    /// read-only because process does not have permission to create a journal
    /// file in the same directory as the database and the creation of a journal
    /// file is a prerequisite for writing.
    ReadonlyDirectory = 1544,
    /// The SQLITE_IOERR_TRUNCATE error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the [VFS](https://www.sqlite.org/vfs.html) layer while trying to truncate a file to a smaller size.
    IoerrTruncate = 1546,
    /// The SQLITE_CANTOPEN_SYMLINK result code is returned by the [sqlite3_open()](https://www.sqlite.org/c3ref/open.html) interface and its siblings when the [SQLITE_OPEN_NOFOLLOW](https://www.sqlite.org/c3ref/c_open_autoproxy.html) flag is used and the database file is a symbolic link.
    CantopenSymlink = 1550,
    /// The SQLITE_CONSTRAINT_PRIMARYKEY error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [PRIMARY KEY constraint](https://www.sqlite.org/lang_createtable.html#primkeyconst) failed.
    ConstraintPrimarykey = 1555,
    /// The SQLITE_IOERR_FSTAT error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the [VFS](https://www.sqlite.org/vfs.html) layer while trying to invoke fstat() (or the equivalent) on a file in order to determine information such as the file size or access permissions.
    IoerrFstat = 1802,
    /// The SQLITE_CONSTRAINT_TRIGGER error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [RAISE function](https://www.sqlite.org/lang_createtrigger.html#raise) within a [trigger](https://www.sqlite.org/lang_createtrigger.html) fired, causing the SQL statement to abort.
    ConstraintTrigger = 1811,
    /// The SQLITE_IOERR_UNLOCK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within xUnlock method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object.
    IoerrUnlock = 2058,
    /// The SQLITE_CONSTRAINT_UNIQUE error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [UNIQUE constraint](https://www.sqlite.org/lang_createtable.html#uniqueconst) failed.
    ConstraintUnique = 2067,
    /// The SQLITE_IOERR_UNLOCK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within xLock method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object while trying to obtain a read lock.
    IoerrRdlock = 2314,
    /// The SQLITE_CONSTRAINT_VTAB error code is not currently used by the SQLite core. However, this error code is available for use by application-defined [virtual tables](https://www.sqlite.org/vtab.html).
    ConstraintVtab = 2323,
    /// The SQLITE_IOERR_UNLOCK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within xDelete method on the [sqlite3_vfs](https://www.sqlite.org/c3ref/vfs.html) object.
    IoerrDelete = 2570,
    /// The SQLITE_CONSTRAINT_ROWID error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that a [rowid](https://www.sqlite.org/lang_createtable.html#rowid) is not unique.
    ConstraintRowid = 2579,
    /// The SQLITE_IOERR_BLOCKED error code is no longer used.
    IoerrBlocked = 2826,
    /// The SQLITE_CONSTRAINT_PINNED error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that an [UPDATE trigger](https://www.sqlite.org/lang_createtrigger.html) attempted do delete the row that was being updated in the middle of the update.
    ConstraintPinned = 2835,
    /// The SQLITE_IOERR_NOMEM error code is sometimes returned by the [VFS](https://www.sqlite.org/vfs.html) layer to indicate that an operation could not be completed due to the inability to allocate sufficient memory. This error code is normally converted into [SQLITE_NOMEM](https://www.sqlite.org/rescode.html#nomem) by the higher layers of SQLite before being returned to the application.
    IoerrNomem = 3082,
    /// The SQLITE_CONSTRAINT_DATATYPE error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_CONSTRAINT](https://www.sqlite.org/rescode.html#constraint) indicating that an insert or update attempted to store a value inconsistent with the column's declared type in a table defined as STRICT.
    ConstraintDatatype = 3091,
    /// The SQLITE_IOERR_ACCESS error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xAccess method on the [sqlite3_vfs](https://www.sqlite.org/c3ref/vfs.html) object.
    IoerrAccess = 3338,
    /// The SQLITE_IOERR_CHECKRESERVEDLOCK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xCheckReservedLock method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object.
    IoerrCheckreservedlock = 3594,
    /// The SQLITE_IOERR_LOCK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error in the advisory file locking logic. Usually an SQLITE_IOERR_LOCK error indicates a problem obtaining a [PENDING lock](https://www.sqlite.org/lockingv3.html#pending_lock). However it can also indicate miscellaneous locking errors on some of the specialized [VFSes](https://www.sqlite.org/vfs.html) used on Macs.
    IoerrLock = 3850,
    /// The SQLITE_IOERR_ACCESS error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xClose method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object.
    IoerrClose = 4106,
    /// The SQLITE_IOERR_DIR_CLOSE error code is no longer used.
    IoerrDirClose = 4362,
    /// The SQLITE_IOERR_SHMOPEN error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xShmMap method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object while trying to open a new shared memory segment.
    IoerrShmopen = 4618,
    /// The SQLITE_IOERR_SHMSIZE error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xShmMap method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object while trying to enlarge a ["shm" file](https://www.sqlite.org/walformat.html#shm) as part of [WAL mode](https://www.sqlite.org/wal.html) transaction processing. This error may indicate that the underlying filesystem volume is out of space.
    IoerrShmsize = 4874,
    /// The SQLITE_IOERR_SHMLOCK error code is no longer used.
    IoerrShmlock = 5130,
    /// The SQLITE_IOERR_SHMMAP error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xShmMap method on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object while trying to map a shared memory segment into the process address space.
    IoerrShmmap = 5386,
    /// The SQLITE_IOERR_SEEK error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xRead or xWrite methods on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object while trying to seek a file descriptor to the beginning point of the file where the read or write is to occur.
    IoerrSeek = 5642,
    /// The SQLITE_IOERR_DELETE_NOENT error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating that the xDelete method on the [sqlite3_vfs](https://www.sqlite.org/c3ref/vfs.html) object failed because the file being deleted does not exist.
    IoerrDeleteNoent = 5898,
    /// The SQLITE_IOERR_MMAP error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating an I/O error within the xFetch or xUnfetch methods on the [sqlite3_io_methods](https://www.sqlite.org/c3ref/io_methods.html) object while trying to map or unmap part of the database file into the process address space.
    IoerrMmap = 6154,
    /// The SQLITE_IOERR_GETTEMPPATH error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) indicating that the [VFS](https://www.sqlite.org/vfs.html) is unable to determine a suitable directory in which to place temporary files.
    IoerrGettemppath = 6410,
    /// The SQLITE_IOERR_CONVPATH error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) used only by Cygwin [VFS](https://www.sqlite.org/vfs.html) and indicating that the cygwin_conv_path() system call failed. See also: [SQLITE_CANTOPEN_CONVPATH](https://www.sqlite.org/rescode.html#cantopen_convpath)
    IoerrConvpath = 6666,
    /// The SQLITE_IOERR_VNODE error code is a code reserved for use by
    /// extensions. It is not used by the SQLite core.
    IoerrVnode = 6922,
    /// The SQLITE_IOERR_AUTH error code is a code reserved for use by
    /// extensions. It is not used by the SQLite core.
    IoerrAuth = 7178,
    /// The SQLITE_IOERR_BEGIN_ATOMIC error code indicates that the underlying operating system reported and error on the [SQLITE_FCNTL_BEGIN_ATOMIC_WRITE](https://www.sqlite.org/c3ref/c_fcntl_begin_atomic_write.html#sqlitefcntlbeginatomicwrite) file-control. This only comes up when [SQLITE_ENABLE_ATOMIC_WRITE](https://www.sqlite.org/compile.html#enable_atomic_write) is enabled and the database is hosted on a filesystem that supports atomic writes.
    IoerrBeginAtomic = 7434,
    /// The SQLITE_IOERR_COMMIT_ATOMIC error code indicates that the underlying operating system reported and error on the [SQLITE_FCNTL_COMMIT_ATOMIC_WRITE](https://www.sqlite.org/c3ref/c_fcntl_begin_atomic_write.html#sqlitefcntlcommitatomicwrite) file-control. This only comes up when [SQLITE_ENABLE_ATOMIC_WRITE](https://www.sqlite.org/compile.html#enable_atomic_write) is enabled and the database is hosted on a filesystem that supports atomic writes.
    IoerrCommitAtomic = 7690,
    /// The SQLITE_IOERR_ROLLBACK_ATOMIC error code indicates that the underlying operating system reported and error on the [SQLITE_FCNTL_ROLLBACK_ATOMIC_WRITE](https://www.sqlite.org/c3ref/c_fcntl_begin_atomic_write.html#sqlitefcntlrollbackatomicwrite) file-control. This only comes up when [SQLITE_ENABLE_ATOMIC_WRITE](https://www.sqlite.org/compile.html#enable_atomic_write) is enabled and the database is hosted on a filesystem that supports atomic writes.
    IoerrRollbackAtomic = 7946,
    /// The SQLITE_IOERR_DATA error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) used only by [checksum VFS shim](https://www.sqlite.org/cksumvfs.html) to indicate that the checksum on a page of the database file is incorrect.
    IoerrData = 8202,
    /// The SQLITE_IOERR_CORRUPTFS error code is an [extended error code](https://www.sqlite.org/rescode.html#pve) for [SQLITE_IOERR](https://www.sqlite.org/rescode.html#ioerr) used only by a VFS to indicate that a seek or read failure was due to the request not falling within the file's boundary rather than an ordinary device failure. This often indicates a corrupt filesystem.
    IoerrCorruptfs = 8458,
}

impl SqliteError {
    /// Returns the last error that occurred.
    pub fn last(conn: SqliteClient) -> Self {
        Self::last_(conn.id()).unwrap_or_default()
    }

    pub(super) fn last_(conn: u64) -> Option<Self> {
        call_host_alloc::<Option<Self>>(|len_ptr| unsafe {
            lunatic_sqlite_api::guest_api::sqlite_guest_bindings::last_error(conn, len_ptr)
        })
        .ok()
        .flatten()
    }

    pub(super) fn from_code(code: u32) -> Option<Self> {
        match SqliteCode::from_code(code) {
            Some(SqliteCode::Ok) => None,
            Some(code) => Some(SqliteError {
                code,
                message: None,
            }),
            None => None,
        }
    }
}

impl Default for SqliteError {
    fn default() -> Self {
        Self {
            code: SqliteCode::Error,
            message: None,
        }
    }
}

impl std::fmt::Display for SqliteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.code, f)?;

        if let Some(msg) = &self.message {
            write!(f, ": {msg}")?;
        }

        Ok(())
    }
}

impl std::error::Error for SqliteError {}

impl SqliteCode {
    pub(super) fn from_code(code: u32) -> Option<Self> {
        match code {
            0 => Some(SqliteCode::Ok),
            1 => Some(SqliteCode::Error),
            2 => Some(SqliteCode::Internal),
            3 => Some(SqliteCode::Perm),
            4 => Some(SqliteCode::Abort),
            5 => Some(SqliteCode::Busy),
            6 => Some(SqliteCode::Locked),
            7 => Some(SqliteCode::Nomem),
            8 => Some(SqliteCode::Readonly),
            9 => Some(SqliteCode::Interrupt),
            10 => Some(SqliteCode::Ioerr),
            11 => Some(SqliteCode::Corrupt),
            12 => Some(SqliteCode::Notfound),
            13 => Some(SqliteCode::Full),
            14 => Some(SqliteCode::Cantopen),
            15 => Some(SqliteCode::Protocol),
            16 => Some(SqliteCode::Empty),
            17 => Some(SqliteCode::Schema),
            18 => Some(SqliteCode::Toobig),
            19 => Some(SqliteCode::Constraint),
            20 => Some(SqliteCode::Mismatch),
            21 => Some(SqliteCode::Misuse),
            22 => Some(SqliteCode::Nolfs),
            23 => Some(SqliteCode::Auth),
            24 => Some(SqliteCode::Format),
            25 => Some(SqliteCode::Range),
            26 => Some(SqliteCode::Notadb),
            27 => Some(SqliteCode::Notice),
            28 => Some(SqliteCode::Warning),
            100 => Some(SqliteCode::Row),
            101 => Some(SqliteCode::Done),
            256 => Some(SqliteCode::OkLoadPermanently),
            257 => Some(SqliteCode::ErrorMissingCollseq),
            261 => Some(SqliteCode::BusyRecovery),
            262 => Some(SqliteCode::LockedSharedcache),
            264 => Some(SqliteCode::ReadonlyRecovery),
            266 => Some(SqliteCode::IoerrRead),
            267 => Some(SqliteCode::CorruptVtab),
            270 => Some(SqliteCode::CantopenNotempdir),
            275 => Some(SqliteCode::ConstraintCheck),
            279 => Some(SqliteCode::AuthUser),
            283 => Some(SqliteCode::NoticeRecoverWal),
            284 => Some(SqliteCode::WarningAutoindex),
            513 => Some(SqliteCode::ErrorRetry),
            516 => Some(SqliteCode::AbortRollback),
            517 => Some(SqliteCode::BusySnapshot),
            518 => Some(SqliteCode::LockedVtab),
            520 => Some(SqliteCode::ReadonlyCantlock),
            522 => Some(SqliteCode::IoerrShortRead),
            523 => Some(SqliteCode::CorruptSequence),
            526 => Some(SqliteCode::CantopenIsdir),
            531 => Some(SqliteCode::ConstraintCommithook),
            539 => Some(SqliteCode::NoticeRecoverRollback),
            769 => Some(SqliteCode::ErrorSnapshot),
            773 => Some(SqliteCode::BusyTimeout),
            776 => Some(SqliteCode::ReadonlyRollback),
            778 => Some(SqliteCode::IoerrWrite),
            779 => Some(SqliteCode::CorruptIndex),
            782 => Some(SqliteCode::CantopenFullpath),
            787 => Some(SqliteCode::ConstraintForeignkey),
            1032 => Some(SqliteCode::ReadonlyDbmoved),
            1034 => Some(SqliteCode::IoerrFsync),
            1038 => Some(SqliteCode::CantopenConvpath),
            1043 => Some(SqliteCode::ConstraintFunction),
            1288 => Some(SqliteCode::ReadonlyCantinit),
            1290 => Some(SqliteCode::IoerrDirFsync),
            1294 => Some(SqliteCode::CantopenDirtywal),
            1299 => Some(SqliteCode::ConstraintNotnull),
            1544 => Some(SqliteCode::ReadonlyDirectory),
            1546 => Some(SqliteCode::IoerrTruncate),
            1550 => Some(SqliteCode::CantopenSymlink),
            1555 => Some(SqliteCode::ConstraintPrimarykey),
            1802 => Some(SqliteCode::IoerrFstat),
            1811 => Some(SqliteCode::ConstraintTrigger),
            2058 => Some(SqliteCode::IoerrUnlock),
            2067 => Some(SqliteCode::ConstraintUnique),
            2314 => Some(SqliteCode::IoerrRdlock),
            2323 => Some(SqliteCode::ConstraintVtab),
            2570 => Some(SqliteCode::IoerrDelete),
            2579 => Some(SqliteCode::ConstraintRowid),
            2826 => Some(SqliteCode::IoerrBlocked),
            2835 => Some(SqliteCode::ConstraintPinned),
            3082 => Some(SqliteCode::IoerrNomem),
            3091 => Some(SqliteCode::ConstraintDatatype),
            3338 => Some(SqliteCode::IoerrAccess),
            3594 => Some(SqliteCode::IoerrCheckreservedlock),
            3850 => Some(SqliteCode::IoerrLock),
            4106 => Some(SqliteCode::IoerrClose),
            4362 => Some(SqliteCode::IoerrDirClose),
            4618 => Some(SqliteCode::IoerrShmopen),
            4874 => Some(SqliteCode::IoerrShmsize),
            5130 => Some(SqliteCode::IoerrShmlock),
            5386 => Some(SqliteCode::IoerrShmmap),
            5642 => Some(SqliteCode::IoerrSeek),
            5898 => Some(SqliteCode::IoerrDeleteNoent),
            6154 => Some(SqliteCode::IoerrMmap),
            6410 => Some(SqliteCode::IoerrGettemppath),
            6666 => Some(SqliteCode::IoerrConvpath),
            6922 => Some(SqliteCode::IoerrVnode),
            7178 => Some(SqliteCode::IoerrAuth),
            7434 => Some(SqliteCode::IoerrBeginAtomic),
            7690 => Some(SqliteCode::IoerrCommitAtomic),
            7946 => Some(SqliteCode::IoerrRollbackAtomic),
            8202 => Some(SqliteCode::IoerrData),
            8458 => Some(SqliteCode::IoerrCorruptfs),
            _ => None,
        }
    }
}

impl std::fmt::Display for SqliteCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SqliteCode::Ok => write!(f, "SQLITE_OK"),
            SqliteCode::Error => write!(f, "SQLITE_ERROR"),
            SqliteCode::Internal => write!(f, "SQLITE_INTERNAL"),
            SqliteCode::Perm => write!(f, "SQLITE_PERM"),
            SqliteCode::Abort => write!(f, "SQLITE_ABORT"),
            SqliteCode::Busy => write!(f, "SQLITE_BUSY"),
            SqliteCode::Locked => write!(f, "SQLITE_LOCKED"),
            SqliteCode::Nomem => write!(f, "SQLITE_NOMEM"),
            SqliteCode::Readonly => write!(f, "SQLITE_READONLY"),
            SqliteCode::Interrupt => write!(f, "SQLITE_INTERRUPT"),
            SqliteCode::Ioerr => write!(f, "SQLITE_IOERR"),
            SqliteCode::Corrupt => write!(f, "SQLITE_CORRUPT"),
            SqliteCode::Notfound => write!(f, "SQLITE_NOTFOUND"),
            SqliteCode::Full => write!(f, "SQLITE_FULL"),
            SqliteCode::Cantopen => write!(f, "SQLITE_CANTOPEN"),
            SqliteCode::Protocol => write!(f, "SQLITE_PROTOCOL"),
            SqliteCode::Empty => write!(f, "SQLITE_EMPTY"),
            SqliteCode::Schema => write!(f, "SQLITE_SCHEMA"),
            SqliteCode::Toobig => write!(f, "SQLITE_TOOBIG"),
            SqliteCode::Constraint => write!(f, "SQLITE_CONSTRAINT"),
            SqliteCode::Mismatch => write!(f, "SQLITE_MISMATCH"),
            SqliteCode::Misuse => write!(f, "SQLITE_MISUSE"),
            SqliteCode::Nolfs => write!(f, "SQLITE_NOLFS"),
            SqliteCode::Auth => write!(f, "SQLITE_AUTH"),
            SqliteCode::Format => write!(f, "SQLITE_FORMAT"),
            SqliteCode::Range => write!(f, "SQLITE_RANGE"),
            SqliteCode::Notadb => write!(f, "SQLITE_NOTADB"),
            SqliteCode::Notice => write!(f, "SQLITE_NOTICE"),
            SqliteCode::Warning => write!(f, "SQLITE_WARNING"),
            SqliteCode::Row => write!(f, "SQLITE_ROW"),
            SqliteCode::Done => write!(f, "SQLITE_DONE"),
            SqliteCode::OkLoadPermanently => write!(f, "SQLITE_OK_LOAD_PERMANENTLY"),
            SqliteCode::ErrorMissingCollseq => write!(f, "SQLITE_ERROR_MISSING_COLLSEQ"),
            SqliteCode::BusyRecovery => write!(f, "SQLITE_BUSY_RECOVERY"),
            SqliteCode::LockedSharedcache => write!(f, "SQLITE_LOCKED_SHAREDCACHE"),
            SqliteCode::ReadonlyRecovery => write!(f, "SQLITE_READONLY_RECOVERY"),
            SqliteCode::IoerrRead => write!(f, "SQLITE_IOERR_READ"),
            SqliteCode::CorruptVtab => write!(f, "SQLITE_CORRUPT_VTAB"),
            SqliteCode::CantopenNotempdir => write!(f, "SQLITE_CANTOPEN_NOTEMPDIR"),
            SqliteCode::ConstraintCheck => write!(f, "SQLITE_CONSTRAINT_CHECK"),
            SqliteCode::AuthUser => write!(f, "SQLITE_AUTH_USER"),
            SqliteCode::NoticeRecoverWal => write!(f, "SQLITE_NOTICE_RECOVER_WAL"),
            SqliteCode::WarningAutoindex => write!(f, "SQLITE_WARNING_AUTOINDEX"),
            SqliteCode::ErrorRetry => write!(f, "SQLITE_ERROR_RETRY"),
            SqliteCode::AbortRollback => write!(f, "SQLITE_ABORT_ROLLBACK"),
            SqliteCode::BusySnapshot => write!(f, "SQLITE_BUSY_SNAPSHOT"),
            SqliteCode::LockedVtab => write!(f, "SQLITE_LOCKED_VTAB"),
            SqliteCode::ReadonlyCantlock => write!(f, "SQLITE_READONLY_CANTLOCK"),
            SqliteCode::IoerrShortRead => write!(f, "SQLITE_IOERR_SHORT_READ"),
            SqliteCode::CorruptSequence => write!(f, "SQLITE_CORRUPT_SEQUENCE"),
            SqliteCode::CantopenIsdir => write!(f, "SQLITE_CANTOPEN_ISDIR"),
            SqliteCode::ConstraintCommithook => write!(f, "SQLITE_CONSTRAINT_COMMITHOOK"),
            SqliteCode::NoticeRecoverRollback => write!(f, "SQLITE_NOTICE_RECOVER_ROLLBACK"),
            SqliteCode::ErrorSnapshot => write!(f, "SQLITE_ERROR_SNAPSHOT"),
            SqliteCode::BusyTimeout => write!(f, "SQLITE_BUSY_TIMEOUT"),
            SqliteCode::ReadonlyRollback => write!(f, "SQLITE_READONLY_ROLLBACK"),
            SqliteCode::IoerrWrite => write!(f, "SQLITE_IOERR_WRITE"),
            SqliteCode::CorruptIndex => write!(f, "SQLITE_CORRUPT_INDEX"),
            SqliteCode::CantopenFullpath => write!(f, "SQLITE_CANTOPEN_FULLPATH"),
            SqliteCode::ConstraintForeignkey => write!(f, "SQLITE_CONSTRAINT_FOREIGNKEY"),
            SqliteCode::ReadonlyDbmoved => write!(f, "SQLITE_READONLY_DBMOVED"),
            SqliteCode::IoerrFsync => write!(f, "SQLITE_IOERR_FSYNC"),
            SqliteCode::CantopenConvpath => write!(f, "SQLITE_CANTOPEN_CONVPATH"),
            SqliteCode::ConstraintFunction => write!(f, "SQLITE_CONSTRAINT_FUNCTION"),
            SqliteCode::ReadonlyCantinit => write!(f, "SQLITE_READONLY_CANTINIT"),
            SqliteCode::IoerrDirFsync => write!(f, "SQLITE_IOERR_DIR_FSYNC"),
            SqliteCode::CantopenDirtywal => write!(f, "SQLITE_CANTOPEN_DIRTYWAL"),
            SqliteCode::ConstraintNotnull => write!(f, "SQLITE_CONSTRAINT_NOTNULL"),
            SqliteCode::ReadonlyDirectory => write!(f, "SQLITE_READONLY_DIRECTORY"),
            SqliteCode::IoerrTruncate => write!(f, "SQLITE_IOERR_TRUNCATE"),
            SqliteCode::CantopenSymlink => write!(f, "SQLITE_CANTOPEN_SYMLINK"),
            SqliteCode::ConstraintPrimarykey => write!(f, "SQLITE_CONSTRAINT_PRIMARYKEY"),
            SqliteCode::IoerrFstat => write!(f, "SQLITE_IOERR_FSTAT"),
            SqliteCode::ConstraintTrigger => write!(f, "SQLITE_CONSTRAINT_TRIGGER"),
            SqliteCode::IoerrUnlock => write!(f, "SQLITE_IOERR_UNLOCK"),
            SqliteCode::ConstraintUnique => write!(f, "SQLITE_CONSTRAINT_UNIQUE"),
            SqliteCode::IoerrRdlock => write!(f, "SQLITE_IOERR_RDLOCK"),
            SqliteCode::ConstraintVtab => write!(f, "SQLITE_CONSTRAINT_VTAB"),
            SqliteCode::IoerrDelete => write!(f, "SQLITE_IOERR_DELETE"),
            SqliteCode::ConstraintRowid => write!(f, "SQLITE_CONSTRAINT_ROWID"),
            SqliteCode::IoerrBlocked => write!(f, "SQLITE_IOERR_BLOCKED"),
            SqliteCode::ConstraintPinned => write!(f, "SQLITE_CONSTRAINT_PINNED"),
            SqliteCode::IoerrNomem => write!(f, "SQLITE_IOERR_NOMEM"),
            SqliteCode::ConstraintDatatype => write!(f, "SQLITE_CONSTRAINT_DATATYPE"),
            SqliteCode::IoerrAccess => write!(f, "SQLITE_IOERR_ACCESS"),
            SqliteCode::IoerrCheckreservedlock => write!(f, "SQLITE_IOERR_CHECKRESERVEDLOCK"),
            SqliteCode::IoerrLock => write!(f, "SQLITE_IOERR_LOCK"),
            SqliteCode::IoerrClose => write!(f, "SQLITE_IOERR_CLOSE"),
            SqliteCode::IoerrDirClose => write!(f, "SQLITE_IOERR_DIR_CLOSE"),
            SqliteCode::IoerrShmopen => write!(f, "SQLITE_IOERR_SHMOPEN"),
            SqliteCode::IoerrShmsize => write!(f, "SQLITE_IOERR_SHMSIZE"),
            SqliteCode::IoerrShmlock => write!(f, "SQLITE_IOERR_SHMLOCK"),
            SqliteCode::IoerrShmmap => write!(f, "SQLITE_IOERR_SHMMAP"),
            SqliteCode::IoerrSeek => write!(f, "SQLITE_IOERR_SEEK"),
            SqliteCode::IoerrDeleteNoent => write!(f, "SQLITE_IOERR_DELETE_NOENT"),
            SqliteCode::IoerrMmap => write!(f, "SQLITE_IOERR_MMAP"),
            SqliteCode::IoerrGettemppath => write!(f, "SQLITE_IOERR_GETTEMPPATH"),
            SqliteCode::IoerrConvpath => write!(f, "SQLITE_IOERR_CONVPATH"),
            SqliteCode::IoerrVnode => write!(f, "SQLITE_IOERR_VNODE"),
            SqliteCode::IoerrAuth => write!(f, "SQLITE_IOERR_AUTH"),
            SqliteCode::IoerrBeginAtomic => write!(f, "SQLITE_IOERR_BEGIN_ATOMIC"),
            SqliteCode::IoerrCommitAtomic => write!(f, "SQLITE_IOERR_COMMIT_ATOMIC"),
            SqliteCode::IoerrRollbackAtomic => write!(f, "SQLITE_IOERR_ROLLBACK_ATOMIC"),
            SqliteCode::IoerrData => write!(f, "SQLITE_IOERR_DATA"),
            SqliteCode::IoerrCorruptfs => write!(f, "SQLITE_IOERR_CORRUPTFS"),
        }
    }
}

pub trait SqliteErrorExt {
    fn into_sqlite_error(self) -> Result<(), SqliteError>;
}

impl SqliteErrorExt for u32 {
    fn into_sqlite_error(self) -> Result<(), SqliteError> {
        match SqliteError::from_code(self) {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
